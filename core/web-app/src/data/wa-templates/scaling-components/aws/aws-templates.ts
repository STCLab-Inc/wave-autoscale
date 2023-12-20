import { TemplateSection } from '../..';
import { EC2_AUTOSCALING_CODE } from './ec2-autoscaling-code';
import { ECS_AUTOASCALING_CODE } from './ecs-autoscaling-code';
import { EMR_EC2_CODE } from './emr-ec2-code';
import { LAMBDA_CONCURRENCY_CODE } from './lambda-concurrency-code';
import { WAF_V2_CODE } from './waf-v2-code.';

// https://gcpicons.com/
export const AWS_TEMPLATE_SECTION: TemplateSection = {
  title: 'AWS',
  description: 'Amazon Web Services',
  templates: [
    {
      title: 'EC2 Auto Scaling',
      description: 'Scaling EC2 instances based on metrics',
      image1: '/assets/app-icons/aws.svg',
      image2: '/assets/app-icons/aws-ec2-autoscaling.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: EC2_AUTOSCALING_CODE,
    },
    {
      title: 'ECS Auto Scaling',
      description: 'Scaling ECS tasks based on metrics',
      image1: '/assets/app-icons/aws.svg',
      image2: '/assets/app-icons/aws-ecs.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: ECS_AUTOASCALING_CODE,
    },
    {
      title: 'Lambda Concurrency',
      description:
        'Adjust reserved concurrency and provisioned concurrency dynamically to archive optimal performance and efficiency',
      image1: '/assets/app-icons/aws.svg',
      image2: '/assets/app-icons/aws-lambda.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: LAMBDA_CONCURRENCY_CODE,
    },
    {
      title: 'EMR on EC2',
      description:
        'Configure the on-demand or spot instances, step concurrency, and more',
      image1: '/assets/app-icons/aws.svg',
      image2: '/assets/app-icons/aws-emr.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: EMR_EC2_CODE,
    },
    {
      title: 'AWS WAF v2',
      description:
        'Adjust the rate limit of a WAF rule dynamically based on metrics',
      image1: '/assets/app-icons/aws.svg',
      image2: '/assets/app-icons/aws-waf.svg',
      url: 'https://kubernetes.io/docs/setup/production-environment/tools/kubeadm/install-kubeadm/',
      code: WAF_V2_CODE,
    },
  ],
};
