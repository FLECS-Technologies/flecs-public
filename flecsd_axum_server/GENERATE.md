# How to generate the flecsd_axum_server code

We use the [rust-axum generator](https://openapi-generator.tech/docs/generators/rust-axum)
from [openapi-generator.tech](https://openapi-generator.tech/) to generate the code for the flecscd-axum-server. The
generator currently does not perfectly match our requirements, and we have to make some manual adjustments to the
mustache templates which control code
generation. All necessary steps are explained below.

## Generate the code

Execute the following command from the repository root directory. Adjust the version if necessary
``docker run --rm -v ${PWD}:/local --user $(id -u):$(id -g) openapitools/openapi-generator-cli:v7.11.0 generate -i /local/api/openapi.yaml --additional-properties=packageName=flecsd_axum_server,generateAliasAsModel=true,packageVersion=2.0.0 -g rust-axum -t /local/flecsd_axum_server/openapi_generator_templates -o /local/flecsd_axum_server``

## Format the code

Format the generated code using [rustfmt](https://github.com/rust-lang/rustfmt). To format doc tests as well, we use
nightly and an additional config option:  ``cargo +nightly fmt -- --config format_code_in_doc_comments=true``

## Apply patches

Some features are infeasible to implement without making manual adjustments after code generation. We therefore save
these changes in patch files to be able to apply them after code generation.
These patches are located in `flecsd_axum_server/patches` and should be applied in order of increasing number.
e.g.
``git apply flecsd_axum_server/patches/0-stream-exports.patch``

## Make necessary manual adjustments

If necessary make manual adjustments to the template files in `flecsd_axum_server/openapi_generator_templates` and
regenerate the code. To get templates for more files you can extract them via
``docker run --rm -v ${PWD}/templates:/out openapitools/openapi-generator-cli:v7.11.0 author template -g rust-axum``
and copy them to `flecsd_axum_server/openapi_generator_templates`.