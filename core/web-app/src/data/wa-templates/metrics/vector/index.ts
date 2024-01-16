import { TemplateSection } from '../..';
import { VECTOR_POSTGRES_CODE } from './vector-postgres-code';
import { VECTOR_PROMETHEUS_CODE } from './vector-prometheus-code';

export const VECTOR_TEMPLATE_SECTION: TemplateSection = {
  title: 'Vector',
  description: 'Vector',
  templates: [
    {
      title: 'Prometheus',
      description: 'Fetch metrics from Prometheus',
      image1: '/assets/app-icons/vector.svg',
      image2: '/assets/app-icons/prometheus.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: VECTOR_PROMETHEUS_CODE,
      metricsOnly: true,
    },
    {
      title: 'Postgres',
      description: 'Fetch metrics from Postgres',
      image1: '/assets/app-icons/vector.svg',
      image2: '/assets/app-icons/postgres.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: VECTOR_POSTGRES_CODE,
      metricsOnly: true,
    },
  ],
};
