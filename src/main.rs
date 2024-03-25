use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use zkp_stark::{*, primefield::*};
use hex;

#[derive(Deserialize)]
struct ProofSubmission {
    proof: String,
}

#[derive(Serialize)]
struct VerificationResponse {
    success: bool,
    message: String,
}

struct FibonacciClaim {
    index: usize,
    value: FieldElement,
}

impl Verifiable for FibonacciClaim {
    fn constraints(&self) -> Constraints {
        use RationalExpression::*;

        // Seed
        let mut seed = self.index.to_be_bytes().to_vec();
        seed.extend_from_slice(&self.value.as_montgomery().to_bytes_be());

        // Constraint repetitions
        let trace_length = self.index.next_power_of_two();
        let g = Constant(FieldElement::root(trace_length).unwrap());
        let on_row = |index| (X - g.pow(index)).inv();
        let every_row = || (X - g.pow(trace_length - 1)) / (X.pow(trace_length) - 1.into());

        let c = Constraints::from_expressions((trace_length, 2), seed, vec![
            (Trace(0, 1) - Trace(1, 0)) * every_row(),
            (Trace(1, 1) - Trace(0, 0) - Trace(1, 0)) * every_row(),
            (Trace(0, 0) - 1.into()) * on_row(0),
            (Trace(0, 0) - (&self.value).into()) * on_row(self.index),
        ])
        .unwrap();
        return c
    }
}

async fn verify_proof(submission: web::Json<ProofSubmission>) -> impl Responder {
    // Deserialize the hex-encoded proof
    let proof_bytes = match hex::decode(&submission.proof) {
        Ok(bytes) => bytes,
        Err(_) => return HttpResponse::BadRequest().json(VerificationResponse {
            success: false,
            message: "Invalid proof format".to_string(),
        }),
    };

        let verification_result = match verify_the_proof(proof_bytes) {
        Ok(_) => VerificationResponse {
            success: true,
            message: "Proof verification successful!".to_string(),
        },
        Err(e) => VerificationResponse {
            success: false,
            message: format!("Proof verification failed: {:?}", e),
        },
    };

    HttpResponse::Ok().json(verification_result)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server is running at http://127.0.0.1:8080/");
    HttpServer::new(|| {
        App::new().service(
            web::resource("/submit_proof").route(web::post().to(verify_proof)),
        )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn verify_the_proof(proof_bytes: Vec<u8>) -> Result<(), &'static str> {
    let proof = zkp_stark::Proof::from_bytes(proof_bytes);

    let claim = FibonacciClaim {
        index: 5000,
        value: FieldElement::from_hex_str("069673d708ad3174714a2c27ffdb56f9b3bfb38c1ea062e070c3ace63e9e26eb")
    };
    match claim.verify(&proof) {
        Ok(_) => Ok(()),
        Err(_) => Err("Verification failed gggggg"),
    }
}

