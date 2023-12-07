import React, { useEffect, useState } from 'react';
import { Controller, useForm } from 'react-hook-form';
import dynamic from 'next/dynamic';
import { YAMLParseError } from 'yaml';
import ScalingPlanService from '@/services/scaling-plan';
import { generateScalingPlanDefinition } from '@/utils/scaling-plan-binding';
import { ScalingPlanDefinitionEx } from '../scaling-plan-definition-ex';
import yaml from 'js-yaml';

// This is a dynamic import that will only be rendered in the client side.
// This is because AceEditor is not compatible with SSR.
const EditorContainerDynamic = dynamic(() => import('./code-editor'), {
  ssr: false,
});

interface ScalingPlanCodeProps {
  scalingPlanCode: string;
}

export default function ScalingPlanCode({
  scalingPlan,
  onChange,
}: {
  scalingPlan: ScalingPlanDefinitionEx;
  onChange: () => void;
}) {
  // react-hook-form
  const { handleSubmit, control, setValue } = useForm<ScalingPlanCodeProps>();
  const [annotations, setAnnotations] = useState<any[]>([]);

  useEffect(() => {
    if (scalingPlan) {
      setValue('scalingPlanCode', yaml.dump(scalingPlan));
    } else {
      setValue('scalingPlanCode', '');
    }
  }, [scalingPlan]);

  const onSubmit = async (data: ScalingPlanCodeProps) => {
    const { scalingPlanCode } = data;
    const scalingPlan = generateScalingPlanDefinition(
      yaml.load(scalingPlanCode) as ScalingPlanDefinitionEx
    );
    try {
      const result = await ScalingPlanService.updateScalingPlan(scalingPlan);
      console.info({ result });
      onChange();
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
          name="scalingPlanCode"
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
