import { TemplateSection } from '../..';
import { AZURE_FUNCTIONS_CODE } from './azure-functions-code';
import { AZURE_VMSS_AUTOSCALING_CODE } from './azure-vmss-autoscaling-code';

export const AZURE_TEMPLATE_SECTION: TemplateSection = {
  title: 'Azure',
  description: 'Microsoft Azure',
  templates: [
    {
      title: 'Azure VM Scale Set Autoscaling',
      description: 'Scaling Azure VM Scale Sets based on metrics',
      image1: '/assets/app-icons/azure.svg',
      image2: '/assets/app-icons/azure-vmss.svg',
      url: 'https://docs.microsoft.com/en-us/azure/azure-monitor/platform/autoscale-common-metrics',
      code: AZURE_VMSS_AUTOSCALING_CODE,
    },
    {
      title: 'Azure Functions ',
      description:
        'Configure the min/max instances of an Azure Function based on metrics',
      image1: '/assets/app-icons/azure.svg',
      image2: '/assets/app-icons/azure-functions.svg',
      url: 'https://docs.microsoft.com/en-us/azure/container-instances/container-instances-scale',
      code: AZURE_FUNCTIONS_CODE,
    },
  ],
};
