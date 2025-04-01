use crate::auth::SignupRequest;
use actix_web::{App, test};

#[actix_rt::test]
async fn test_signup() {
    let conn = todo!();
    let app = test::init_service(App::new()).await;

    // Create test data for signup
    let new_user = SignupRequest {
        email: "test@example.com".to_string(),
        password: "password".to_string(),
    };

    // Call the signup endpoint
    let req = test::TestRequest::post()
        .uri("/signup")
        .set_json(&new_user)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().as_u16(), 200);
}
