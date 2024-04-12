use packets::value::ValueType;
use packets::{ClientCommandExecutor, ServerResponse};
use server::state;

#[tokio::test]
async fn test_empty_incr() {
    let state = state::State::default();
    let incr_result = state.incr("test".into()).await;
    assert!(matches!(
        incr_result,
        Ok(ServerResponse::Single {
            value: ValueType::Int(1)
        })
    ));
}
