// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { ObjectKind } from "./object-kind";
import type { PlanItemDefinition } from "./plan-item-definition";

export interface ScalingPlanDefinition { kind: ObjectKind, db_id: string, id: string, metadata: object, plans: Array<PlanItemDefinition>, }