import { TemplateSection } from '../..';
import { NOT_PREPARED_CODE } from '../not-prepared-code';
import { NETFUNNEL_CODE } from './netfunnel-code';

export const TRAFFIC_TEMPLATE_SECTION: TemplateSection = {
  title: 'Traffic Services',
  description: 'Traffic management',
  templates: [
    {
      title: 'NetFunnel',
      description:
        'Protect your services and be in control during traffic surges',
      image1: '/assets/app-icons/netfunnel.png',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: NETFUNNEL_CODE,
    },
    {
      title: 'Cloudflare Rule',
      description:
        'Set up a Cloudflare rule to protect your services from traffic surges',
      image1: '/assets/app-icons/cloudflare.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: NOT_PREPARED_CODE,
    },
  ],
};
