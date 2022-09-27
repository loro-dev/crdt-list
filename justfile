fuzz TARGET *FLAGS:
  cargo fuzz run {{TARGET}} -- -max_total_time=60 {{FLAGS}}

test *FLAGS:
  RUST_BACKTRACE=full cargo nextest run {{FLAGS}}
