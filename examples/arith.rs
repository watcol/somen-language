use somen::{call, prelude::*};
use somen_language::{
    infix,
    numeric::{integer::integer, signed},
};

fn main() {
    futures::executor::block_on(async {
        let mut stream = stream::from_iter("-1*(3+4)-4*3/6".chars())
            .positioned::<usize>()
            .buffered_rewind();
        let res = arith().complete().parse(&mut stream).await;
        println!("{:?}", res);
    })
}

fn arith<'a, I: Input<Ok = char> + ?Sized + 'a>() -> impl Parser<I, Output = i32> + 'a {
    infix! { expr: i32;
        val: signed(|neg| integer(10, neg), false) => val;
        val: call!(arith).between(token('('), token(')')) => val;

        @[prefix(x)]
        '-' => -x;

        @[left(x, y)]
        '*' => x * y;
        '/' => x / y;

        @[left(x, y)]
        '+' => x + y;
        '-' => x - y;
    }
}
