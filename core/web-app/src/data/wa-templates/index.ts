import METRICS_TEMPLATE_SECTIONS from './metrics';
import SCALING_COMPONENT_TEMPLATES from './scaling-components';

export interface Template {
  title: string;
  description: string;
  image1: string;
  image2?: string;
  url: string;
  code: string;
  metricsOnly?: boolean;
}

export interface TemplateSection {
  title: string;
  description: string;
  templates: Template[];
}

export default {
  SCALING_COMPONENTS: SCALING_COMPONENT_TEMPLATES,
  METRICS: METRICS_TEMPLATE_SECTIONS,
};
