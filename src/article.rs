use std::{path::PathBuf, rc::Rc, sync::LazyLock};

use chrono::NaiveDate;
use lol_html::{RewriteStrSettings, element, end_tag, html_content::ContentType, rewrite_str};
use syntect::{
    highlighting::{Theme, ThemeSet},
    html::highlighted_html_for_string,
    parsing::SyntaxSet,
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub(crate) struct SeoMeta {
    pub thumbnail: Option<&'static str>,
    pub description: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub(crate) struct ArticleMeta {
    pub date: NaiveDate,
    pub date_str: String,
    pub title: String,
    pub slug: String,
    pub filename: String,
    pub content: String,
    pub signature_to_display: u8,
    pub next_article_title: String,
    pub next_article_slug: String,
    pub article_uri: &'static str,
    pub seo_meta: SeoMeta,
}

impl ArticleMeta {
    pub fn new(
        date: NaiveDate,
        title: String,
        filename: String,
        content: String,
        signature_to_display: u8,
        next_article_title: String,
        next_article_slug: String,
        seo_meta: SeoMeta,
    ) -> Self {
        let date_str = date.format("%Y-%m-%d").to_string();
        let slug = filename[..filename.len() - 5].to_string();
        let article_uri = format!("articles/{}", slug).leak();
        Self {
            date,
            date_str,
            title,
            slug,
            filename,
            content,
            signature_to_display,
            next_article_title,
            next_article_slug,
            seo_meta,
            article_uri
        }
    }
}

pub static ARTICLES: LazyLock<&'static [ArticleMeta]> = LazyLock::new(load_articles);

pub const NUM_SIGNATURES: u8 = 3;

fn load_articles() -> &'static [ArticleMeta] {
    let syntax_set = Rc::new(SyntaxSet::load_defaults_newlines());
    let theme_path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "src/themes/codeblock_theme.md"]
        .iter()
        .collect();
    let highlighting_theme = Rc::new(
        ThemeSet::get_theme(theme_path)
            .expect("Theme path exists")
            .clone(),
    );
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "templates/articles"]
        .iter()
        .collect();
    if let Ok(articles) = std::fs::read_dir(path) {
        let mut metas = Vec::new();
        for article in articles {
            if let Ok(path) = article
                && let Ok(filename) = path.file_name().into_string()
                && let Ok(content) = std::fs::read_to_string(path.path())
            {
                let date =
                    NaiveDate::parse_from_str(&filename[0..10], "%Y-%m-%d").unwrap_or_default();
                let slug = &filename;
                let title = &filename[11..filename.len() - 5].replace("_", " ");
                let (content, seo_metadata) = extract_seo_metadata(&content);
                let highlighted_content = highlight_codeblocks(
                    Rc::new(content.to_string()),
                    syntax_set.clone(),
                    highlighting_theme.clone(),
                );
                let article_meta = ArticleMeta::new(
                    date,
                    title.to_string(),
                    slug.to_string(),
                    highlighted_content,
                    0,              // signature_to_display
                    "".to_string(), // next_article_title
                    "".to_string(), // next_article_slug,
                    seo_metadata,
                );
                metas.push(article_meta);
            }
        }
        metas.sort_by_key(|a| a.date);
        metas.reverse();
        let default_article_meta = ArticleMeta::default();
        let first_meta = metas.last().unwrap_or(&default_article_meta);
        let (mut next_article_title, mut next_article_slug) =
            (first_meta.title.clone(), first_meta.slug.clone());
        for (index, article_meta) in metas.iter_mut().enumerate() {
            let signature_to_display = index as u8 % NUM_SIGNATURES;
            article_meta.signature_to_display = signature_to_display;
            article_meta.next_article_title = next_article_title;
            article_meta.next_article_slug = next_article_slug;
            next_article_title = article_meta.title.clone();
            next_article_slug = article_meta.slug.clone();
        }
        metas.leak()
    } else {
        &[]
    }
}

pub fn get_article_metas() -> &'static [ArticleMeta] {
    &ARTICLES
}

fn extract_seo_metadata(html: &str) -> (&str, SeoMeta) {
    let trimmed = html.trim_start();
    let mut seo_meta = SeoMeta {
        thumbnail: None,
        description: None,
    };
    if let Some(rest) = trimmed.strip_prefix("<!--")
        && let Some(end) = rest.find("-->")
    {
        let meta: Vec<(&str, &str)> = rest[..end]
            .lines()
            .filter_map(|l| l.split_once(':'))
            .map(|(k, v)| (k.trim(), v.trim()))
            .collect();

        for (meta_key, meta_value) in meta {
            match meta_key {
                "description" => seo_meta.description = Some(meta_value.to_string().leak()),
                "thumbnail" => seo_meta.thumbnail = Some(meta_value.to_string().leak()),
                _ => {}
            }
        }
        (&rest[end + 3..], seo_meta)
    } else {
        (html, SeoMeta::default())
    }
}

fn highlight_codeblocks(html: Rc<String>, ss: Rc<SyntaxSet>, theme: Rc<Theme>) -> String {
    rewrite_str(
        &html.clone(),
        RewriteStrSettings::new().append_element_content_handler(element!("div[data-paint]", {
            move |el| {
                let theme_clone = theme.clone();
                let ss_clone = ss.clone();
                let html_clone = html.clone();

                // read the language denoted with data-paint attribute aka [data-paint="html"] for example
                let paint_lang = el.get_attribute("data-paint").unwrap_or("rust".to_string());

                // get location of the content's start
                let inner_start = el.source_location().bytes().end;

                // nuke the innerHTML
                el.set_inner_content("", ContentType::Html);

                el.on_end_tag(end_tag!(move |end| {
                    // get the content's end location
                    let inner_end = end.source_location().bytes().start;

                    // get the whole content from original string now that we know where to snip it out
                    let inner = &html_clone[inner_start..inner_end];

                    // use syntect to color our codeblocks
                    let syntax = ss_clone
                        .find_syntax_by_token(&paint_lang)
                        .unwrap_or_else(|| ss_clone.find_syntax_plain_text());
                    let transformed =
                        highlighted_html_for_string(inner, &ss_clone, syntax, &theme_clone)
                            .expect("highlighting failed");

                    // insert the highlighted codeblock back into the element, completing the swap
                    end.before(&transformed, ContentType::Html);
                    end.before(
                        &format!(
                            "<span class='lang-name'>{}</span>",
                            prettify_lang(&paint_lang)
                        )
                        .to_string(),
                        ContentType::Html,
                    );

                    Ok(())
                }))?;
                Ok(())
            }
        })),
    )
    .unwrap()
}

fn prettify_lang(lang: &str) -> &str {
    match lang {
        "sh" => "shell",
        _ => lang,
    }
}
