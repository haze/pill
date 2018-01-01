<img src="https://cdn.rawgit.com/hvze/pill/7563861d/images/pill_logo.svg" width=55/> 

# pill

The (Pretty) Ill Programming Language's Rust Interpreter

<img src="https://cdn.rawgit.com/hvze/pill/2310b6b1/images/fizzbuzz.png" />

## Getting Started

1. Clone the repo and cd.
`git clone https://github.com/hvze/pill & cd pill`
2. Build (or run) the project.

`cargo build --release & ./target/release/pill <files>`
or 
`cargo run -- <flags> <files>`


### Prerequisites

I'm like 100% sure you need to have the rust toolchain installed, if you need to do that, try [rustup.rs](https://rustup.rs)

### Installing

clone the repository and run `cargo build --release` to produce a binary, then copy to your PATH variable for use elsewhere.

`git clone https://github.com/hvze/pill && cd pill && cargo run -- tests/programs/do_good.ill`

```Finished dev [unoptimized + debuginfo] target(s) in 0.0 secs
Running `target/debug/pill tests/programs/do/do_good.ill`
 a_res = 1
 PILL Execution took: 0s, (1 ms)
```

## Running the tests

### Categorized tests

Most of the tests are pretty organized, just run the interpreter with the `-d` flag and see what the output is and if you see anything suspicious or unexpected output, let me know. Migration to a test suite will come soon!


```
pill -d <test_file>.ill
```

## Deployment

Clone the repository and run `cargo build --release` to produce a binary, then copy to your PATH variable for use elsewhere.

## Built With

The following rust crates:
* clap, v2.25.0
* time, v0.1.37
* termcolor, v0.3.3
* pcre, v0.2.3
* either, v1.2.0

## Contributing

Just make a pull request, and I'll review it. No Code of Conduct or anything similar here.

## Versioning

Versioning is a bit hectic as I haven't been keeping tags or marks on what i've added, but after the website is released i will start doing versioned releaes.

## Authors

* **Haze Booth** - *Initial work* - [hvze](https://github.com/hvze)

## License

This project is licensed under the MIT License

## Acknowledgments

* Hat tip to anyone who's code was used
* Inspired loosely by FASM 

