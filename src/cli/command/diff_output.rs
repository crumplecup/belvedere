//! Diff command output option parsing.

use tracing::instrument;

use crate::BelvedereError;

use super::{BelvedereCli, BelvedereCliOutput};

impl BelvedereCli {
    #[instrument(level = "debug", skip(output))]
    pub(in crate::cli::command) fn parse_diff_output_arg(
        arguments: &[String],
        index: usize,
        output: &mut BelvedereCliOutput,
    ) -> Result<Option<usize>, BelvedereError> {
        if let Some(consumed) = Self::parse_diff_certificate_output_arg(arguments, index, output)? {
            return Ok(Some(consumed));
        }
        if let Some(consumed) = Self::parse_diff_previous_output_arg(arguments, index, output)? {
            return Ok(Some(consumed));
        }
        if let Some(consumed) = Self::parse_diff_current_output_arg(arguments, index, output)? {
            return Ok(Some(consumed));
        }
        if let Some(consumed) =
            Self::parse_diff_current_projection_output_arg(arguments, index, output)?
        {
            return Ok(Some(consumed));
        }
        Self::parse_diff_alias_output_arg(arguments, index, output)
    }

    #[instrument(level = "debug", skip(output))]
    fn parse_diff_certificate_output_arg(
        arguments: &[String],
        index: usize,
        output: &mut BelvedereCliOutput,
    ) -> Result<Option<usize>, BelvedereError> {
        let consumed = match arguments[index].as_str() {
            "--certificate-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCertificateJson;
                0
            }
            "--certificate-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCertificateFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--run-certificate-json" => {
                *output = BelvedereCliOutput::LandscapeDiffRunCertificateJson;
                0
            }
            "--run-certificate-file" => {
                *output = BelvedereCliOutput::LandscapeDiffRunCertificateFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--manifest-json" => {
                *output = BelvedereCliOutput::LandscapeDiffManifestJson;
                0
            }
            "--manifest-file" => {
                *output = BelvedereCliOutput::LandscapeDiffManifestFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--diff-json" => {
                *output = BelvedereCliOutput::LandscapeDiffJson;
                0
            }
            "--diff-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffReceiptJson;
                0
            }
            "--diff-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffReceiptFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            _ => return Ok(None),
        };

        Ok(Some(consumed))
    }

    #[instrument(level = "debug", skip(output))]
    fn parse_diff_previous_output_arg(
        arguments: &[String],
        index: usize,
        output: &mut BelvedereCliOutput,
    ) -> Result<Option<usize>, BelvedereError> {
        let consumed = match arguments[index].as_str() {
            "--previous-registry-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousRegistryReceiptJson;
                0
            }
            "--previous-registry-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousRegistryReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--previous-assessment-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousAssessmentReceiptJson;
                0
            }
            "--previous-assessment-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousAssessmentReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--previous-landscape-json" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousLandscapeJson;
                0
            }
            "--previous-landscape-file" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousLandscapeFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--previous-landscape-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousLandscapeReceiptJson;
                0
            }
            "--previous-landscape-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousLandscapeReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--previous-view-model-json" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousViewModelJson;
                0
            }
            "--previous-view-model-file" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousViewModelFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--previous-filter-facets-json" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousFilterFacetsJson;
                0
            }
            "--previous-filter-facets-file" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousFilterFacetsFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--previous-source-navigation-json" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousSourceNavigationJson;
                0
            }
            "--previous-source-navigation-file" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousSourceNavigationFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--previous-view-model-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousViewModelReceiptJson;
                0
            }
            "--previous-view-model-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousViewModelReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--previous-ui-ir-json" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousUiIrJson;
                0
            }
            "--previous-ui-ir-file" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousUiIrFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--previous-ui-ir-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousUiIrReceiptJson;
                0
            }
            "--previous-ui-ir-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousUiIrReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            _ => return Ok(None),
        };

        Ok(Some(consumed))
    }

    #[instrument(level = "debug", skip(output))]
    fn parse_diff_current_output_arg(
        arguments: &[String],
        index: usize,
        output: &mut BelvedereCliOutput,
    ) -> Result<Option<usize>, BelvedereError> {
        let consumed = match arguments[index].as_str() {
            "--current-registry-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentRegistryReceiptJson;
                0
            }
            "--current-registry-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentRegistryReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--current-assessment-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentAssessmentReceiptJson;
                0
            }
            "--current-assessment-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentAssessmentReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--current-landscape-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentLandscapeJson;
                0
            }
            "--current-landscape-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentLandscapeFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--current-landscape-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentLandscapeReceiptJson;
                0
            }
            "--current-landscape-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentLandscapeReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            _ => return Ok(None),
        };

        Ok(Some(consumed))
    }

    #[instrument(level = "debug", skip(output))]
    fn parse_diff_current_projection_output_arg(
        arguments: &[String],
        index: usize,
        output: &mut BelvedereCliOutput,
    ) -> Result<Option<usize>, BelvedereError> {
        let consumed = match arguments[index].as_str() {
            "--current-view-model-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentViewModelJson;
                0
            }
            "--current-view-model-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentViewModelFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--current-filter-facets-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentFilterFacetsJson;
                0
            }
            "--current-filter-facets-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentFilterFacetsFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--current-source-navigation-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentSourceNavigationJson;
                0
            }
            "--current-source-navigation-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentSourceNavigationFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--current-view-model-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentViewModelReceiptJson;
                0
            }
            "--current-view-model-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentViewModelReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--current-ui-ir-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentUiIrJson;
                0
            }
            "--current-ui-ir-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentUiIrFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--current-ui-ir-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentUiIrReceiptJson;
                0
            }
            "--current-ui-ir-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentUiIrReceiptFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            _ => return Ok(None),
        };

        Ok(Some(consumed))
    }

    #[instrument(level = "debug", skip(output))]
    fn parse_diff_alias_output_arg(
        arguments: &[String],
        index: usize,
        output: &mut BelvedereCliOutput,
    ) -> Result<Option<usize>, BelvedereError> {
        let consumed = match arguments[index].as_str() {
            "--landscape-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffLandscapeReceiptJson;
                0
            }
            "--landscape-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffLandscapeReceiptFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--landscape-json" => {
                *output = BelvedereCliOutput::LandscapeDiffLandscapeJson;
                0
            }
            "--landscape-file" => {
                *output = BelvedereCliOutput::LandscapeDiffLandscapeFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--view-model-json" => {
                *output = BelvedereCliOutput::LandscapeDiffViewModelJson;
                0
            }
            "--view-model-file" => {
                *output = BelvedereCliOutput::LandscapeDiffViewModelFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--filter-facets-json" => {
                *output = BelvedereCliOutput::LandscapeDiffFilterFacetsJson;
                0
            }
            "--filter-facets-file" => {
                *output = BelvedereCliOutput::LandscapeDiffFilterFacetsFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--source-navigation-json" => {
                *output = BelvedereCliOutput::LandscapeDiffSourceNavigationJson;
                0
            }
            "--source-navigation-file" => {
                *output = BelvedereCliOutput::LandscapeDiffSourceNavigationFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--review-workflow-json" => {
                *output = BelvedereCliOutput::LandscapeDiffReviewWorkflowJson;
                0
            }
            "--review-workflow-file" => {
                *output = BelvedereCliOutput::LandscapeDiffReviewWorkflowFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--view-model-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffViewModelReceiptJson;
                0
            }
            "--view-model-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffViewModelReceiptFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--ui-ir-json" => {
                *output = BelvedereCliOutput::LandscapeDiffUiIrJson;
                0
            }
            "--ui-ir-file" => {
                *output =
                    BelvedereCliOutput::LandscapeDiffUiIrFile(Self::next_path(arguments, index)?);
                1
            }
            "--ui-ir-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffUiIrReceiptJson;
                0
            }
            "--ui-ir-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffUiIrReceiptFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            _ => return Ok(None),
        };

        Ok(Some(consumed))
    }
}
