#[allow(warnings)]
mod bindings;

use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use bindings::exports::component::numbat_component::{
    self,
    numbat::{Guest, GuestCtx},
};
use numbat::{
    module_importer::BuiltinModuleImporter, pretty_print::PrettyPrint, InterpreterSettings,
};

struct Component;

struct ComponentCtx {
    inner: RefCell<numbat::Context>,
}

impl GuestCtx for ComponentCtx {
    fn new() -> Self {
        let mut inner = numbat::Context::new(BuiltinModuleImporter::default());
        let mut settings = InterpreterSettings {
            print_fn: Box::new(move |_| {}),
        };
        let _ = inner.interpret_with_settings(
            &mut settings,
            "use prelude",
            numbat::resolver::CodeSource::Internal,
        );
        ComponentCtx {
            inner: RefCell::new(inner),
        }
    }

    fn eval(&self, input: String) -> Result<String, String> {
        let format_for_irc = true;
        let to_be_printed: Arc<Mutex<Vec<_>>> = Arc::new(Mutex::new(vec![]));
        let to_be_printed_c = to_be_printed.clone();

        let mut ctx = self.inner.borrow_mut();

        let mut settings = numbat::InterpreterSettings {
            print_fn: Box::new(move |s| {
                to_be_printed_c.lock().unwrap().push(s.clone());
            }),
        };

        let registry = ctx.dimension_registry().clone();

        let (statements, result) = ctx
            .interpret_with_settings(&mut settings, &input, numbat::resolver::CodeSource::Text)
            .map_err(|e| e.to_string())?;

        let mut s = String::new();
        for statement in &statements {
            let markup = statement.pretty_print();
            if format_for_irc {
                s.push_str(&numbat::markup::Formatter::format(
                    &IRCFormatter,
                    &markup,
                    false,
                ));
            } else {
                s.push_str(markup.to_string().as_str());
            }
            // s.push_str(markup.to_string().as_str());
            // s.push(numbat_component::numbat::Markup::new(markup));
        }

        let r = result.to_markup(statements.last(), &registry, true, true);
        if format_for_irc {
            s.push_str(&numbat::markup::Formatter::format(&IRCFormatter, &r, false));
        } else {
            s.push_str(r.to_string().as_str());
        }
        // s.push_str(r.to_string().as_str());

        Ok(s.trim().to_string())
    }
}

// impl GuestMarkup for numbat::markup::Markup {
//     fn to_string(&self) -> String {
//         ToString::to_string(&self)
//     }

//     fn to_irc_string(&self) -> String {
//         numbat::markup::Formatter::format(&IRCFormatter, &self, false)

//     }
// }

struct IRCFormatter;

impl numbat::markup::Formatter for IRCFormatter {
    fn format_part(
        &self,
        numbat::markup::FormattedString(_output_type, format_type, text):  &numbat::markup::FormattedString,
    ) -> numbat::compact_str::CompactString {
        let text = match text {
            numbat::markup::CompactStrCow::Owned(s) => format!("{s}"),
            numbat::markup::CompactStrCow::Static(s) => format!("{s}"),
        };
        match format_type {
            numbat::markup::FormatType::Whitespace => format!("{text}"),
            numbat::markup::FormatType::Emphasized => format!("\x02{text}\x0f"),
            numbat::markup::FormatType::Dimmed => format!("{text}"),
            numbat::markup::FormatType::Text => format!("{text}"),
            numbat::markup::FormatType::String => format!("\x0303{text}\x0f"),
            numbat::markup::FormatType::Keyword => format!("\x0313{text}\x0f"),
            numbat::markup::FormatType::Value => format!("\x0308{text}\x0f"),
            numbat::markup::FormatType::Unit => format!("\x0311{text}\x0f"),
            numbat::markup::FormatType::Identifier => format!("{text}"),
            numbat::markup::FormatType::TypeIdentifier => format!("\x0312\x1d{text}\x0f"),
            numbat::markup::FormatType::Operator => format!("\x02{text}\x0f"),
            numbat::markup::FormatType::Decorator => format!("\x0303{text}\x0f"),
        }
        .into()
    }
}

impl Guest for Component {
    type Ctx = ComponentCtx;

    // type Markup = numbat::markup::Markup;

    // fn format_for_irc(input: numbat_component::numbat::Markup) -> String {
    //     let inner: &numbat::markup::Markup = input.get();

    //     numbat::markup::Formatter::format(&IRCFormatter, &inner, false)
    // }
}

bindings::export!(Component with_types_in bindings);
