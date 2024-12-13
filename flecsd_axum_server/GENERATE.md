# How to generate the flecsd_axum_server code

We use the [rust-axum generator](https://openapi-generator.tech/docs/generators/rust-axum)
from [openapi-generator.tech](https://openapi-generator.tech/) to generate the code for the flecscd-axum-server. The
generator currently does not perfectly match our requirements, and we have to make some manual adjustments after the
code
generation. All necessary steps are explained below.

## Generate the code

Execute the following command from the repository root directory. Adjust the version if necessary
``docker run --rm -v ${PWD}:/local openapitools/openapi-generator-cli:v7.8.0 generate -i /local/flecs/api/openapi.yaml --additional-properties=packageName=flecsd_axum_server,generateAliasAsModel=true,packageVersion=2.0.0 -g rust-axum -o /local/flecsd-axum-server/``

## Format the code

Format the generated code using [rustfmt](https://github.com/rust-lang/rustfmt). To format doc tests as well, we use
nightly and an additional config option:  ``cargo +nightly fmt -- --config format_code_in_doc_comments=true``

## Apply patch

Apply the patch found in flecsd_axum_server/generated.patch ``git apply flecsd_axum_server/generated.patch``

## Make necessary manual adjustments

If necessary make manual code adjustments, format the code and add the changes to flecsd_axum_server/generated.patch