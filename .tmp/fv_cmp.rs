use bingo_core::types::FactValue;
fn main(){
 let a=FactValue::Integer(1);
 let b=FactValue::Integer(2);
 println!("a>b => {}", a>b);
}
