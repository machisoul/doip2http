use crate::doip_client::DoipClient;

pub struct UdsClient {
  doip_client: Option<DoipClient>,
}
