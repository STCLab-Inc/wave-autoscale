import React, { useEffect, useState } from 'react';
import { Controller, useForm } from 'react-hook-form';
import { YAMLParseError } from 'yaml';
import ScalingPlanService from '@/services/scaling-plan';
import { generateScalingPlanDefinition } from '@/utils/scaling-plan-binding';
import { ScalingPlanDefinitionEx } from '../scaling-plan-definition-ex';
import yaml from 'js-yaml';
import dynamic from 'next/dynamic';

const YAMLEditor = dynamic(() => import('../../common/yaml-editor'), {
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
      <div className="textarea-bordered textarea textarea-md h-full w-full rounded-none border-none p-0">
        <Controller
          control={control}
          name="scalingPlanCode"
          render={({ field: { onChange, value } }) => (
            <YAMLEditor
              value={value}
              onChange={onChange}
              annotations={annotations}
            />
          )}
        />
      </div>
    </form>
  );
}
