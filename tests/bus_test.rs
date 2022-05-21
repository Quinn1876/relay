/**
 *  @test Bus test
 *  @brief The bus test is designed to test the integration of the entire system.
 *  It includes:
 *  - A mock desktop which implements project butterfree,
 *  - A virtual can device which runs a single torchic node which will emit values every 200ms.
 * */
mod common;

#[test]
fn simple_test() {
  /* Just here to check compilation temporarily */
  let x = 1 + 1;
  assert_eq!(1, 1);

}
