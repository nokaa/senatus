use hayaku::{self, Request, Response, Status};

use super::Context;
use database as db;
use post::Post;

use std::str::FromStr;

pub fn home_handler(_req: &Request, res: &mut Response, ctx: &Context) {
    info!("home handler");
    let tmpl_ctx = &ctx.config;
    let result = ctx.templates.render("home", &tmpl_ctx).unwrap();
    debug!("{}", result);
    res.body(result.as_bytes());
}

pub fn install_page_handler(_req: &Request, res: &mut Response, ctx: &Context) {
    info!("install_page handler");
    let tmpl_ctx = &ctx.config;
    let result = ctx.templates.render("install", &tmpl_ctx).unwrap();
    debug!("{}", result);
    res.body(result.as_bytes());
}

pub fn install_handler(_req: &Request, res: &mut Response, ctx: &Context) {
    info!("install handler");
}

pub fn board_handler(req: &Request, res: &mut Response, ctx: &Context) {
    info!("board handler");
    let params = hayaku::get_path_params(req);
    let board = &params["board"];
    info!("board: {}", board);

    let pool = &ctx.db_pool;
    let board = if let Ok(Some(b)) = db::get_board(pool.clone(), board) {
        b
    } else {
        return not_found_handler(req, res, ctx);
    };

    let catalog = if let Ok(Some(c)) = db::get_catalog(pool.clone(), &board) {
        c
    } else {
        Vec::new()
    };


    let result = ctx.templates.render("board", &(board, catalog)).unwrap();
    debug!("{}", result);
    res.body(result.as_bytes());
}

pub fn new_thread_handler(req: &Request, res: &mut Response, ctx: &Context) {
    info!("new thread handler");
    let params = hayaku::get_path_params(req);
    let board = &params["board"];
    let name = req.form_value("name").unwrap_or("Anonymous".to_string());
    let subject = req.form_value("subject").unwrap_or("".to_string());
    let email = req.form_value("email").unwrap_or("".to_string());
    let content = req.form_value("content").unwrap_or("".to_string());

    // TODO(nokaa): We should also check that the content doesn't contain
    // only whitespace. Otherwise the user could just write a space and achieve
    // the same result.
    if content == "" {
        // TODO(nokaa): Return some sort of error here telling the
        // user that they need to have content to create a post.
        return not_found_handler(req, res, ctx);
    }
    let name = if name == "" {
        "Anonymous".to_string()
    } else {
        name
    };

    let pool = &ctx.db_pool;
    // Make sure that board exists
    let board_exists = db::board_exists(pool.clone(), board);
    if board_exists.is_err() || !board_exists.unwrap() {
        return not_found_handler(req, res, ctx);
    }

    // Write to database
    let thread = Post {
        post_number: 0,
        parent: 0,
        board: board.clone(),
        subject: Some(subject),
        name: name,
        email: email,
        content: content,
    };

    match db::create_thread(pool.clone(), thread) {
        Ok(pnum) => {
            res.redirect(Status::Found,
                         format!("/b/{}/{}", board, pnum),
                         b"Thread created!")
        }
        Err(e) => {
            info!("Unable to create thread!");
            info!("error: {}", e);
            not_found_handler(req, res, ctx);
        }
    }
}

pub fn thread_handler(req: &Request, res: &mut Response, ctx: &Context) {
    info!("thread handler: {}", req.path());
    let params = hayaku::get_path_params(req);
    let board_name = &params["board"];
    let thread_number = &params["thread"];
    let thread_number = if let Ok(t) = i64::from_str(thread_number) {
        t
    } else {
        info!("Error converting to i64!");
        return not_found_handler(req, res, ctx);
    };

    let pool = &ctx.db_pool;
    let board = if let Ok(Some(b)) = db::get_board(pool.clone(), board_name) {
        b
    } else {
        info!("board {} not found!", board_name);
        return not_found_handler(req, res, ctx);
    };

    let thread = if let Ok(t) = db::get_thread(pool.clone(), board_name, thread_number) {
        t
    } else {
        info!("thread {} not found!", thread_number);
        return not_found_handler(req, res, ctx);
    };

    let result = ctx.templates.render("thread", &(board, thread)).unwrap();
    debug!("{}", result);
    res.body(result.as_bytes());
}

pub fn new_post_handler(req: &Request, res: &mut Response, ctx: &Context) {
    info!("new thread handler");
    let params = hayaku::get_path_params(req);
    let board = &params["board"];
    let thread_number = &params["thread"];
    let thread_number = if let Ok(t) = i64::from_str(thread_number) {
        t
    } else {
        info!("Error converting to i64!");
        return not_found_handler(req, res, ctx);
    };
    let name = req.form_value("name").unwrap_or("Anonymous".to_string());
    let email = req.form_value("email").unwrap_or("".to_string());
    let content = req.form_value("content").unwrap_or("".to_string());

    // TODO(nokaa): We should also check that the content doesn't contain
    // only whitespace. Otherwise the user could just write a space and achieve
    // the same result.
    if content == "" {
        // TODO(nokaa): Return some sort of error here telling the
        // user that they need to have content to create a post.
        return not_found_handler(req, res, ctx);
    }
    let name = if name == "" {
        "Anonymous".to_string()
    } else {
        name
    };

    let pool = &ctx.db_pool;
    // Make sure that board exists
    let board_exists = db::board_exists(pool.clone(), board);
    if board_exists.is_err() || !board_exists.unwrap() {
        info!("Board {} does not exist!", board);
        return not_found_handler(req, res, ctx);
    }

    // Make sure that thread exists
    let thread_exists = db::thread_exists(pool.clone(), board, thread_number);
    if thread_exists.is_err() || !thread_exists.unwrap() {
        info!("thread {} does not exist!", thread_number);
        return not_found_handler(req, res, ctx);
    }

    let post = Post {
        post_number: 0,
        parent: thread_number,
        board: board.clone(),
        subject: None,
        name: name,
        email: email,
        content: content,
    };

    if let Err(e) = db::create_post(pool.clone(), post) {
        info!("Unable to create post!");
        info!("error: {}", e);
        return not_found_handler(req, res, ctx);
    } else {
        res.redirect(Status::Found,
                     // format!("/b/{}/{}", board, thread_number),
                     req.path(),
                     b"Post created!");
    }
}

pub fn not_found_handler(_req: &Request, res: &mut Response, ctx: &Context) {
    info!("not found handler");
    let result = ctx.templates.render("404", &()).unwrap();
    debug!("{}", result);
    res.status(Status::NotFound);
    res.body(result.as_bytes());
}
