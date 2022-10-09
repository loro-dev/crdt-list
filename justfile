fuzz TARGET *FLAGS:
  cargo fuzz run {{TARGET}} -- -max_total_time=60 {{FLAGS}}

test *FLAGS:
  RUST_BACKTRACE=full cargo nextest run --features=fuzzing {{FLAGS}}

quick-fuzz:
  cargo fuzz run woot -- -max_total_time=1 &&\
  cargo fuzz run woot-10 -- -max_total_time=1 &&\
  cargo fuzz run yata -- -max_total_time=1
