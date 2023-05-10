import { getInterfaceKeyTypes, MetadataKeyType } from './metadata-key-type';
import { EC2AutoscalingMetadata } from '@/types/bindings/ec2-autoscaling';
import { K8sDeploymentMetadata } from '@/types/bindings/k8s-deployment';
import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';

export type ScalingComponentKeyType = {
  componentName: string;
  keyTypes: MetadataKeyType[];
};

export function getScalingComponentKeyTypes(): ScalingComponentKeyType[] {
  const componentKeyTypes: ScalingComponentKeyType[] = [];

  // EC2 Autoscaling
  const ec2autoscalingComponent = getInterfaceKeyTypes<EC2AutoscalingMetadata>({
    region: '',
    access_key: '',
    secret_key: '',
    asg_name: '',
  });
  componentKeyTypes.push({
    componentName: 'aws-ec2-autoscaling',
    keyTypes: ec2autoscalingComponent,
  });

  // k8s Deployment
  const k8sDeploymentComponent = getInterfaceKeyTypes<K8sDeploymentMetadata>({
    api_server_endpoint: '',
    namespace: '',
    name: '',
  });
  componentKeyTypes.push({
    componentName: 'kubernetes-deployment',
    keyTypes: k8sDeploymentComponent,
  });

  return componentKeyTypes;
}

export function generateScalingComponentDefinition({
  id,
  db_id,
  component_kind,
  metadata,
}: {
  id: string;
  db_id?: string;
  component_kind: string;
  metadata: any;
}) {
  return {
    kind: 'ScalingComponent',
    id,
    db_id,
    component_kind,
    metadata,
  } as ScalingComponentDefinition;
}
