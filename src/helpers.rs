pub fn tg_fail(e: telegram_bot::Error) -> failure::Error { failure::err_msg(format!("{}", e)) }
