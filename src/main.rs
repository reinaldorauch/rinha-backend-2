use std::time::SystemTime;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, http::StatusCode};
use serde_derive::{Deserialize, Serialize};
use std::{sync::Mutex, vec};

#[derive(Default, Debug, Clone, Copy, Serialize)]
struct Client {
    id: u64,
    limite: i128,
    saldo: i128,
}

#[derive(Serialize, Clone)]
enum TransactionType {
    Credit,
    Debit,
}

#[derive(Serialize,Clone)]
struct Transaction {
    client_id: u64,
    value: i128,
    r#type: TransactionType,
    when: SystemTime,
}

#[derive(Deserialize)]
struct TransactionInfo {
    valor: i128,
    tipo: String,
    descricao: String,
}

#[derive(Clone, Serialize, Debug, Default)]
struct TransactionResponse {
    limite: i128,
    saldo: i128,
}

struct AppState {
    clients: Mutex<Vec<Client>>,
    transactions: Mutex<Vec<Transaction>>,
}

enum TransactionError {
    ClientNotFound,
    InvalidTransactionType,
    BelowAllowedLimite
}

impl AppState {

    fn new() -> AppState {
        AppState {
            clients: Mutex::new(vec![
                Client {
                    id: 1,
                    limite: 100000,
                    saldo: 0,
                },
                Client {
                    id: 2,
                    limite: 80000,
                    saldo: 0,
                },
                Client {
                    id: 3,
                    limite: 1000000,
                    saldo: 0,
                },
                Client {
                    id: 4,
                    limite: 10000000,
                    saldo: 0,
                },
                Client {
                    id: 5,
                    limite: 500000,
                    saldo: 0,
                },
            ]),
            transactions: Mutex::new(Vec::new()),
        }
    }

    fn handle_transaction(&self, client_id: u64, transaction: &TransactionInfo) -> Result<TransactionResponse, TransactionError> {
        let mut client_list = self.clients.lock().unwrap();
        let mut transaction_list = self.transactions.lock().unwrap();
        let client = match client_list.binary_search_by_key(&client_id, |c| c.id) {
            Ok(index) => match client_list.get_mut(index) {
                Some(c) => c,
                None => return Err(TransactionError::ClientNotFound)
            },
            Err(_) => return Err(TransactionError::ClientNotFound),
        };

        match transaction.tipo.as_str() {
            "c" => {
                client.saldo += transaction.valor;
                transaction_list.push(Transaction {
                    client_id: client_id,
                    value: transaction.valor,
                    when: SystemTime::now(),
                    r#type: TransactionType::Credit
                });
                Ok(TransactionResponse {
                    limite: client.limite,
                    saldo: client.saldo
                })   
            },
            "d" => {
                if (0 - client.limite) > (client.saldo - transaction.valor) {
                    return Err(TransactionError::BelowAllowedLimite);
                }

                client.saldo -= transaction.valor;
                transaction_list.push(Transaction {
                    client_id: client_id,
                    value: transaction.valor,
                    when: SystemTime::now(),
                    r#type: TransactionType::Debit
                });

                Ok(TransactionResponse {
                    limite: client.limite,
                    saldo: client.saldo
                })
            },
            _ => return Err(TransactionError::InvalidTransactionType)
        }
    }

    fn get_extrato(&self, client_id: u64) -> Result<ExtratoResponse, TransactionError> {
        let clients = self.clients.lock().unwrap();
        let transactions = self.transactions.lock().unwrap();

        let cliente = match clients.binary_search_by_key(&client_id, |c| c.id) {
            Ok(index) => match clients.get(index) {
                Some(c) => c,
                None => return Err(TransactionError::ClientNotFound)
            },
            Err(_) => return Err(TransactionError::ClientNotFound),
        };

        Ok(ExtratoResponse {
            cliente: cliente.clone(),
            ultimas_transacoes: transactions.iter().filter(|t| t.client_id == (client_id)).map(|t| t.clone()).collect()
        })
    }
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/clientes")]
async fn list_clientes(app_state: web::Data<AppState>) -> impl Responder {
    let clients = app_state.clients.lock().unwrap();
    HttpResponse::Ok().json(clients.clone())
}

#[post("/clientes/{client_id}/transacoes")]
async fn transacoes(
    client_id: web::Path<u64>,
    transaction_info: web::Json<TransactionInfo>,
    app_state: web::Data<AppState>,
) -> impl Responder {
    match app_state.handle_transaction(*client_id, &transaction_info) {
        Ok(r) => HttpResponse::Ok().json(r),
        Err(e) => match e {
            TransactionError::ClientNotFound => HttpResponse::NotFound().body("Cliente não encontrado"),
            TransactionError::InvalidTransactionType => HttpResponse::BadRequest().body("Tipo de transação inválida"),
            TransactionError::BelowAllowedLimite => HttpResponse::build(StatusCode::from_u16(422).unwrap()).body("limite indisponível"),
        }
    }
}

#[derive(Serialize)]
struct ExtratoResponse {
    cliente: Client,
    ultimas_transacoes: Vec<Transaction>,
}

#[get("/clientes/{client_id}/extrato")]
async fn extrato(client_id: web::Path<u64>, app_state: web::Data<AppState>) -> impl Responder {
    match app_state.get_extrato(client_id.into_inner()) {
        Ok(extrato) => HttpResponse::Ok().json(extrato),
        Err(e) => match e {
            TransactionError::ClientNotFound => HttpResponse::NotFound().body("Cliente não encontrado"),
            _ => HttpResponse::InternalServerError().body("")
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState::new());

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(hello)
            .service(list_clientes)
            .service(transacoes)
            .service(extrato)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
