use tokio::sync::mpsc;
use tokio::io::{self, AsyncBufReadExt};

pub mod services {
    tonic::include_proto!("services");
}

use services::{
    chat_service_client::ChatServiceClient, 
    ChatMessage, 
    payment_service_client::PaymentServiceClient, 
    PaymentRequest, 
    transaction_service_client::TransactionServiceClient, 
    TransactionRequest
};
use tonic::transport::Channel;
use tokio_stream::wrappers::ReceiverStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut payment_client = PaymentServiceClient::connect("http://[::1]:50051").await?;
    let request_payment = tonic::Request::new(PaymentRequest {
        user_id: "user_123".to_string(),
        amount: 100.0,
    });
    let response_payment = payment_client.process_payment(request_payment).await?;
    println!("Payment Response: {:?}", response_payment.into_inner());

    let mut transaction_client = TransactionServiceClient::connect("http://[::1]:50051").await?;
    let request_transaction = tonic::Request::new(TransactionRequest {
        user_id: "user_123".to_string(),
    });
    let mut transaction_stream = transaction_client.get_transaction_history(request_transaction).await?.into_inner();
    while let Some(transaction) = transaction_stream.message().await? {
        println!("Transaction: {:?}", transaction);
    }

    let channel = Channel::from_static("http://[::1]:50051").connect().await?;
    let mut chat_client = ChatServiceClient::new(channel);
    let (tx, rx) = mpsc::channel(32);

    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            if line.trim().is_empty() {
                continue;
            }
            let message = ChatMessage {
                user_id: "user_123".to_string(),
                message: line,
            };

            if tx_clone.send(message).await.is_err() {
                eprintln!("Failed to send message to server.");
                break;
            }
        }
    });

    let request_chat = tonic::Request::new(ReceiverStream::new(rx));
    let mut response_stream = chat_client.chat(request_chat).await?.into_inner();
    while let Some(response) = response_stream.message().await? {
        println!("Server says: {:?}", response);
    }

    Ok(())
}
