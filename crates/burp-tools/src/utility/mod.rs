use super::{BurpTools, DecoderInput, decoder_json};
use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};

#[tool_router(router = utility_router, vis = "pub(crate)")]
impl BurpTools {
    #[tool(
        name = "decoder",
        description = "One bounded offline decoder tool with many operations. Supply operation for one transform, steps for a recipe, query to search the operation catalog, describe for one operation's metadata, or magic=true for deterministic decode suggestions.",
        annotations(
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn decoder(&self, Parameters(input): Parameters<DecoderInput>) -> String {
        decoder_json(input)
    }
}
