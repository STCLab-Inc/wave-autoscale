'use client';

import React, { useEffect, useState } from 'react';
import { Controller, useForm } from 'react-hook-form';
import dynamic from 'next/dynamic';
import { useParams } from 'next/navigation';
import { YAMLParseError } from 'yaml';

import { usePlanStore } from '../../plan-store';

// This is a dynamic import that will only be rendered in the client side.
// This is because AceEditor is not compatible with SSR.
const EditorContainerDynamic = dynamic(() => import('./editor-container'), {
  ssr: false,
});

export default function PlanningDetailCodePage() {
  const { id: scalingPlanId } = useParams();
  // react-hook-form
  const { handleSubmit, control, setValue } = useForm();
  const [annotations, setAnnotations] = useState<any[]>([]);
  // Plan data management
  const load = usePlanStore((state) => state.load);
  const getYAMLCode = usePlanStore((state) => state.getYAMLCode);
  const applyYAMLCode = usePlanStore((state) => state.applyYAMLCode);

  // If scalingPlanId changes, fetch the scaling plan then it updates plans and nodes.
  useEffect(() => {
    const fetch = async (id: string) => {
      await load(id);
      const yamlCode = await getYAMLCode();
      setValue('expression', yamlCode);
    };

    if (Array.isArray(scalingPlanId)) {
      fetch(scalingPlanId[0]);
    } else {
      fetch(scalingPlanId);
    }
  }, [scalingPlanId]);

  const onSubmit = async (data: any) => {
    const { expression } = data;
    setAnnotations([]);
    try {
      await applyYAMLCode(expression);
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
          name="expression"
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
