import { TemplateSection } from '../..';
import { CLOUD_FUNCTIONS_AUTOSCALING_CODE } from './cloud-functions-autoscaling';
import { CLOUD_RUN_AUTOSCALING_CODE } from './cloud-run-autoscaling-code';
import { MIG_AUTOSCALING_CODE } from './mig-autoscaling-code';

export const GCP_TEMPLATE_SECTION: TemplateSection = {
  title: 'GCP',
  description: 'Google Cloud Platform',
  templates: [
    {
      title: 'Managed Instance Group Autoscaling',
      description: 'Scaling GKE nodes based on metrics',
      image1: '/assets/app-icons/gcp.svg',
      image2: '/assets/app-icons/gcp-compute-engine.svg',
      url: 'https://cloud.google.com/kubernetes-engine/docs/how-to/cluster-autoscaler',
      code: MIG_AUTOSCALING_CODE,
    },
    {
      title: 'Cloud Run Autoscaling',
      description: 'Scaling Cloud Run services based on metrics',
      image1: '/assets/app-icons/gcp.svg',
      image2: '/assets/app-icons/gcp-cloud-run.svg',
      url: 'https://cloud.google.com/run/docs/configuring/max-instances',
      code: CLOUD_RUN_AUTOSCALING_CODE,
    },
    {
      title: 'Cloud Functions Autoscaling',
      description:
        'Adjust min/max instances, max request per instance of a Cloud Function based on metrics',
      image1: '/assets/app-icons/gcp.svg',
      image2: '/assets/app-icons/gcp-cloud-functions.svg',
      url: 'https://cloud.google.com/functions/docs/max-instances',
      code: CLOUD_FUNCTIONS_AUTOSCALING_CODE,
    },
  ],
};
