import { TemplateSection } from '..';
import { AWS_TEMPLATE_SECTION } from './aws/aws-templates';
import { AZURE_TEMPLATE_SECTION } from './azure';
import { GCP_TEMPLATE_SECTION } from './gcp';
import { K8S_TEMPLATE_SECTION } from './k8s';
import { TRAFFIC_TEMPLATE_SECTION } from './traffic';

const SCALING_COMPONENT_TEMPLATES: TemplateSection[] = [
  K8S_TEMPLATE_SECTION,
  AWS_TEMPLATE_SECTION,
  GCP_TEMPLATE_SECTION,
  AZURE_TEMPLATE_SECTION,
  TRAFFIC_TEMPLATE_SECTION,
];

export default SCALING_COMPONENT_TEMPLATES;
