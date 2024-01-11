import { getInterfaceKeyTypes, MetadataKeyType } from './metadata-key-type';
import { EC2AutoscalingMetadata } from '@/types/bindings/ec2-autoscaling';
import { EC2AutoscalingPlanMetadata } from '@/types/bindings/ec2-autoscaling-plan';
import { K8sDeploymentMetadata } from '@/types/bindings/k8s-deployment';
import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';
import { K8sDeploymentPlanMetadata } from '../types/bindings/k8s-deployment-plan';
import JSYaml from 'js-yaml';

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

// For ScalingPlan
export function getScalingComponentPlanKeyTypes(): ScalingComponentKeyType[] {
  const componentKeyTypes: ScalingComponentKeyType[] = [];

  // EC2 Autoscaling
  const ec2autoscalingComponentPlan =
    getInterfaceKeyTypes<EC2AutoscalingPlanMetadata>({
      desired: 0,
      min: 0,
      max: 0,
    });
  componentKeyTypes.push({
    componentName: 'aws-ec2-autoscaling',
    keyTypes: ec2autoscalingComponentPlan,
  });

  // k8s Deployment
  const k8sDeploymentComponent =
    getInterfaceKeyTypes<K8sDeploymentPlanMetadata>({
      replicas: 0,
    });
  componentKeyTypes.push({
    componentName: 'kubernetes-deployment',
    keyTypes: k8sDeploymentComponent,
  });

  return componentKeyTypes;
}

export function generateScalingComponentDefinition({
  kind,
  id,
  db_id,
  component_kind,
  enabled,
  metadata,
}: {
  kind: string;
  id: string;
  db_id?: string;
  component_kind: string;
  enabled: boolean;
  metadata: any;
}) {
  return {
    kind: kind,
    id,
    db_id,
    component_kind,
    enabled,
    metadata: metadata ?? {},
  } as ScalingComponentDefinition;
}

export function serializeScalingComponentDefinition(
  componentDefinition: ScalingComponentDefinition
): string {
  const { kind, id, component_kind, enabled, metadata } = componentDefinition;
  const serialized = JSYaml.dump({
    kind,
    id,
    component_kind,
    enabled,
    metadata,
  });
  return serialized;
}

export function serializeScalingComponentDefinitions(
  componentDefinitions: ScalingComponentDefinition[]
): string {
  const serialized = componentDefinitions.map((componentDefinition) =>
    serializeScalingComponentDefinition(componentDefinition)
  );
  const result = serialized.join('\n---\n');
  return result;
}

export function deserializeScalingComponentDefinitions(
  serialized: string
): ScalingComponentDefinition[] {
  let deserialized: ScalingComponentDefinition[] = JSYaml.loadAll(
    serialized
  ) as ScalingComponentDefinition[];
  // Filter out invalid component definitions
  deserialized = deserialized.filter(
    (definition) => definition !== null
  );
  return deserialized;
}
