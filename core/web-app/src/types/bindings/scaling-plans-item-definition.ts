// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.

export interface ScalingPlansItemDefinition {
  id: string;
  description: string | null;
  expression: string | null;
  cron_expression: string | null;
  cool_down: bigint | null;
  priority: number;
  scaling_components: Array<any>;
  ui: any;
}
