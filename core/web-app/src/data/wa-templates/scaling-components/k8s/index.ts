import { TemplateSection } from '../..';
import { NOT_PREPARED_CODE } from '../not-prepared-code';
import { ISTIO_RETRY_CODE } from './istio-retry-code';
import { ISTIO_TRAFFIC_DELAY_CODE } from './istio-traffic-delay-code';
import { COREDNS_AUTOSCALING_CODE } from './k8s-coredns-autoscaling-code';
import { DEPLOYMENT_REPLICAS_CODE } from './k8s-deployment-replicas-code';

export const K8S_TEMPLATE_SECTION: TemplateSection = {
  title: 'Kubernetes',
  description:
    'Kubernetes is an open-source system for automating deployment, scaling, and management of containerized applications.',
  templates: [
    {
      title: 'Deployment Replicas',
      description: 'Scaling a number of replicas of a deployment with metrics',
      image1: '/assets/app-icons/kubernetes.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: DEPLOYMENT_REPLICAS_CODE,
    },
    {
      title: 'CoreDNS Autoscaling',
      description: 'Scaling CoreDNS pods based on metrics',
      image1: '/assets/app-icons/kubernetes.svg',
      image2: '/assets/app-icons/coredns.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: COREDNS_AUTOSCALING_CODE,
    },
    {
      title: 'Pod Right Sizing',
      description: 'Dynamically right-sizing pods based on metrics',
      image1: '/assets/app-icons/kubernetes.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: NOT_PREPARED_CODE,
    },
    {
      title: 'Istio Traffic Delay',
      description:
        'When a service or a database is heavyly loaded, delay can be used to protect it from overload',
      image1: '/assets/app-icons/kubernetes.svg',
      image2: '/assets/app-icons/istio.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: ISTIO_TRAFFIC_DELAY_CODE,
    },
    {
      title: 'Istio Rate Limits',
      description:
        'When a service or a database is heavyly loaded, rate limiting can be used to protect it from overload',
      image1: '/assets/app-icons/kubernetes.svg',
      image2: '/assets/app-icons/istio.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: ISTIO_RETRY_CODE,
    },
  ],
};
