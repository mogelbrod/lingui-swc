use lingui_macro_plugin::{DescriptorFields, LinguiOptions};

#[macro_use]
mod common;

to!(
    should_transform_define_message,
    r#"
        import { defineMessage, msg } from '@lingui/macro';
        const message1 = defineMessage({
          comment: "Description",
          message: "Message"
        })
        const message2 = msg({
          comment: "Description",
          message: "Message"
        })
     "#
);

to!(
    define_message_should_support_template_literal,
    r#"
        import { defineMessage, msg } from '@lingui/macro';
        const message1 = defineMessage`Message`;
        const message2 = msg`Message`
     "#
);

to!(
    should_preserve_custom_id,
    r#"
        import { defineMessage, plural, arg } from '@lingui/macro';
        const message = defineMessage({
          comment: "Description",
          id: "custom.id",
          message: "Message",
        })
     "#
);

to!(
    should_expand_values,
    r#"
        import { defineMessage, plural, arg } from '@lingui/macro';
        const message = defineMessage({
          message: `Hello ${name}`
        })
     "#
);

to!(
    should_expand_macros,
    r#"
        import { defineMessage, plural, arg } from '@lingui/macro';
        const message = defineMessage({
          comment: "Description",
          message: plural(count, { one: "book", other: "books" })
        })
     "#
);

to!(
    id_only_should_keep_only_id,
    LinguiOptions {
        descriptor_fields: DescriptorFields::IdOnly,
        ..Default::default()
    },
    r#"
        import { defineMessage } from '@lingui/macro'
        const message1 = defineMessage`Message`;
        const message2 = defineMessage({
            message: `Hello ${name}`,
            id: 'msgId',
            comment: 'description for translators',
            context: 'My Context',
        })
    "#
);

to!(
    message_should_keep_message_and_context,
    LinguiOptions {
        descriptor_fields: DescriptorFields::Message,
        ..Default::default()
    },
    r#"
        import { defineMessage } from '@lingui/macro'
        const message1 = defineMessage`Message`;
        const message2 = defineMessage({
            message: `Hello ${name}`,
            id: 'msgId',
            comment: 'description for translators',
            context: 'My Context',
        })
    "#
);

#[test]
fn debug_logs_define_message_and_msg_macros() {
    let source = common::dedent(
        r#"
            import { defineMessage, msg } from '@lingui/macro';
            const first = defineMessage({ message: "Hello", context: "ctx" })
            const second = msg`World`
        "#,
    );

    let (_output, logs) = common::transform_with_debug_logs(source.as_str(), |comments, source_map| {
        swc_core::ecma::visit::fold_pass(lingui_macro_plugin::LinguiMacroFolder::new(
            LinguiOptions {
                debug: true,
                ..Default::default()
            },
            Some(comments.clone()),
            lingui_macro_plugin::DebugSourceMap::from_test(source_map),
        ))
    })
    .expect("Transform produced unexpected errors");

    assert!(logs[0].starts_with("input.tsx:2: defineMessage id=\""));
    assert!(logs[0].contains(" context=\"ctx\""));
    assert!(logs[0].contains(" message=\"Hello\""));
    assert!(logs[1].starts_with("input.tsx:3: msg id=\""));
    assert!(logs[1].contains(" message=\"World\""));
}
