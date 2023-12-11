import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';

export interface ScalingComponentDefinitionEx
  extends ScalingComponentDefinition {
  isChecked: boolean;
  created_at: string;
  updated_at: string;
}
