import { TemplateSection } from '../..';
import { TELEGRAF_CLOUDWATCH_CODE } from './telegraf-cloudwatch-code';
import { TELEGRAF_GCP_MONITORING_CODE } from './telegraf-gcp-monitoring-code';
import { TELEGRAF_PROMETHEUS_CODE } from './telegraf-prometheus-code';

export const TELEGRAF_TEMPLATE_SECTION: TemplateSection = {
  title: 'Telegraf',
  description: 'Telegraf',
  templates: [
    {
      title: 'Prometheus',
      description: 'Fetch metrics from Prometheus',
      image1: '/assets/app-icons/telegraf.svg',
      image2: '/assets/app-icons/prometheus.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: TELEGRAF_PROMETHEUS_CODE,
      metricsOnly: true,
    },
    {
      title: 'AWS CloudWatch',
      description: 'Fetch metrics from CloudWatch',
      image1: '/assets/app-icons/telegraf.svg',
      image2: '/assets/app-icons/aws-cloudwatch.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: TELEGRAF_CLOUDWATCH_CODE,
      metricsOnly: true,
    },
    {
      title: 'Google Cloud Monitoring',
      description: 'Fetch metrics from Google Cloud Monitoring',
      image1: '/assets/app-icons/telegraf.svg',
      image2: '/assets/app-icons/gcp-cloud-monitoring.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: TELEGRAF_GCP_MONITORING_CODE,
      metricsOnly: true,
    },
  ],
};
