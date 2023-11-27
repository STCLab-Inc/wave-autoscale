'use client';

import React, { useEffect, useState } from 'react';
import { Controller, useForm } from 'react-hook-form';
import dynamic from 'next/dynamic';
import { produce } from 'immer';
import { YAMLParseError } from 'yaml';

import ScalingPlanService from '@/services/scaling-plan';
import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import { generateScalingPlanDefinition } from '@/utils/scaling-plan-binding';

interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
}

// This is a dynamic import that will only be rendered in the client side.
// This is because AceEditor is not compatible with SSR.
const EditorContainerDynamic = dynamic(() => import('./code-editor'), {
  ssr: false,
});

export default function ScalingPlansCode({
  scalingPlansItem,
  setScalingPlansItem,
  detailsModalFlag,
  setDetailsModalFlag,
  setFetchFlag,
}: {
  scalingPlansItem?: ScalingPlanDefinitionEx | undefined;
  setScalingPlansItem: (
    scalingPlan: ScalingPlanDefinitionEx | undefined
  ) => void;
  detailsModalFlag: boolean;
  setDetailsModalFlag: (detailsModalFlag: boolean) => void;
  setFetchFlag: (fetchFlag: boolean) => void;
}) {
  // react-hook-form
  const { handleSubmit, control, setValue } = useForm();
  const [annotations, setAnnotations] = useState<any[]>([]);

  const yaml = require('js-yaml');

  const fetchScalingPlans = async () => {
    const response = await ScalingPlanService.getScalingPlans();
    setScalingPlansItem(
      response[
        response.findIndex(
          (scalingPlan: ScalingPlanDefinitionEx) =>
            scalingPlan.db_id === scalingPlansItem?.db_id
        )
      ]
    );
  };

  useEffect(() => {
    if (scalingPlansItem) {
      const { kind, db_id, id, metadata, plans, ...rest } = scalingPlansItem;
      setValue('kind', kind);
      setValue('db_id', db_id);
      setValue('id', id);
      setValue('metadata', yaml.dump(metadata));
      setValue('plans', yaml.dump(plans));
      const newScalingPlanDefinition: ScalingPlanDefinitionEx = {
        kind,
        db_id,
        id,
        metadata,
        plans,
      };
      const scalingPlanForCode = produce(
        newScalingPlanDefinition,
        (draft: any) => {
          draft.plans?.forEach((plan: any) => {
            delete plan.ui;
            plan.expression && !plan.cron_expression
              ? delete plan.cron_expression
              : null;
            plan.cron_expression && !plan.expression
              ? delete plan.expression
              : null;
          });
        }
      );
      setValue('scaling_plan', yaml.dump(scalingPlanForCode));
    }
  }, [scalingPlansItem]);

  const onSubmit = async (data: any) => {
    const { kind, db_id, id, metadata, plans, scaling_plan, ...rest } = data;
    const scalingPlansItem = generateScalingPlanDefinition(
      yaml.load(data.scaling_plan)
    );
    try {
      const result = await ScalingPlanService.updateScalingPlan(
        scalingPlansItem
      );
      console.info({ result });
      fetchScalingPlans();
      setFetchFlag(true);
    } catch (error: any) {
      const { message, linePos } = error as YAMLParseError;
      if (linePos?.[0] && message) {
        setAnnotations([
          {
            row: linePos?.[0].line - 1,
            column: linePos?.[0].col,
            text: message,
            type: 'error',
          },
        ]);
      }
    }
  };

  return (
    <form
      onSubmit={handleSubmit(onSubmit)}
      className="flex h-full w-full flex-col"
    >
      <div className="flex border-b px-4 py-4">
        <button className="left-4 top-4 flex h-8 items-center justify-center rounded-md border border-gray-600 pl-5 pr-5 text-sm text-gray-600">
          UPDATE
        </button>
      </div>

      <div className="textarea-bordered textarea textarea-md h-full w-full rounded-none border-none p-4">
        <Controller
          control={control}
          name="scaling_plan"
          render={({ field: { onChange, value } }) => (
            <EditorContainerDynamic
              onChange={onChange}
              value={value}
              annotations={annotations}
            />
          )}
        />
      </div>
    </form>
  );
}
