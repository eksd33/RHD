use std::io::{self,BufRead, BufReader};

pub fn read_input(stdin_set: bool, file: &str){
    let buffer = BufReader::new;
    if stdin_set{
        for line in io::stdin().lock().lines(){

        }
    }
}