'use client';

import React, { useEffect, useState } from 'react';
import { Controller, useForm } from 'react-hook-form';
import dynamic from 'next/dynamic';
import { YAMLParseError, parse as parseYaml } from 'yaml';
import { produce } from 'immer';

import { ScalingPlanDefinition } from '@/types/bindings/scaling-plan-definition';
import PlanService from '@/services/plan';

interface ScalingPlanDefinitionEx extends ScalingPlanDefinition {
  metadata: { cool_down: number; interval: number; title: string };
}

// This is a dynamic import that will only be rendered in the client side.
// This is because AceEditor is not compatible with SSR.
const EditorContainerDynamic = dynamic(() => import('./code-editor'), {
  ssr: false,
});

export default function PlanningCodeComponent({
  plansItem,
  setPlansItem,
  detailsModalFlag,
  setDetailsModalFlag,
  setFetchFlag,
}: {
  plansItem?: ScalingPlanDefinitionEx | undefined;
  setPlansItem: (plan: ScalingPlanDefinitionEx | undefined) => void;
  detailsModalFlag: boolean;
  setDetailsModalFlag: (detailsModalFlag: boolean) => void;
  setFetchFlag: (fetchFlag: boolean) => void;
}) {
  // react-hook-form
  const { handleSubmit, control, setValue } = useForm();
  const [annotations, setAnnotations] = useState<any[]>([]);

  const yaml = require('js-yaml');

  const fetchPlans = async () => {
    const response = await PlanService.getPlans();
    setPlansItem(
      response[
        response.findIndex(
          (plan: ScalingPlanDefinitionEx) => plan.db_id === plansItem?.db_id
        )
      ]
    );
  };

  useEffect(() => {
    const yamlCode = yaml.dump(plansItem);
    const parsedYaml = parseYaml(yamlCode);
    const { kind, db_id, id, metadata, plans, ...rest } = parsedYaml;
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
    setValue('plan', yaml.dump(scalingPlanForCode));
  }, [plansItem]);

  const onSubmit = async (data: any) => {
    const { yamlCode } = data;

    try {
      const parsedYaml = await parseYaml(yamlCode);
      const { kind, db_id, id, metadata, plans, ...rest } = parsedYaml;
      const newScalingPlanDefinition: ScalingPlanDefinitionEx = {
        kind,
        db_id,
        id,
        metadata,
        plans,
      };
      const result = await PlanService.updatePlan(newScalingPlanDefinition);
      fetchPlans();
      setFetchFlag(true);
      setValue('yamlCode', yaml.dump(plansItem));

      console.info({ result });
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
          name="plan"
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
