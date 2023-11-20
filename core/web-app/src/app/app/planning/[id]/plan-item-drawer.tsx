'use client';

import React, { useEffect, useRef } from 'react';
import { useParams } from 'next/navigation';
import { Controller, useFieldArray, useForm } from 'react-hook-form';
import Image from 'next/image';
import { omit } from 'lodash';
// import 'ace-builds/src-noconflict/ext-language_tools';
import AceEditor from 'react-ace';
import 'ace-builds/src-noconflict/mode-javascript';
import 'ace-builds/src-noconflict/snippets/javascript';
import 'ace-builds/src-noconflict/theme-xcode';

import { PlanItemDefinition } from '@/types/bindings/plan-item-definition';
import { usePlanStore } from '../plan-store';
import ScalingComponentPlanSelect from './scaling-component-plan-select';

export default function PlanItemDrawer({
  planItemDefinition,
}: {
  planItemDefinition?: PlanItemDefinition;
}) {
  const { register, handleSubmit, control, reset, setValue } = useForm();

  const { fields, append, remove } = useFieldArray({
    control,
    name: 'scaling_components',
  });

  const topElement = useRef<HTMLDivElement>(null);
  const { id: scalingPlanId } = useParams();
  const clearSelectedPlan = usePlanStore((state) => state.clearSelection);
  const updatePlanItem = usePlanStore((state) => state.updatePlanItem);
  const removePlanItem = usePlanStore((state) => state.removePlanItem);

  useEffect(() => {
    reset({
      scaling_components: [],
    });
    if (!planItemDefinition) {
      return;
    }
    const { id, description, priority, expression, scaling_components } =
      planItemDefinition;
    setValue('id', id);
    setValue('description', description);
    setValue('priority', priority);
    setValue('expression', expression);
    // setValue('scaling_components', scaling_components || []);
    scaling_components?.forEach((scalingComponent, index) => {
      append(scalingComponent);
    });
    // WORKAROUND: scroll to top
    setTimeout(() => {
      topElement?.current?.scroll({
        top: 0,
      });
    }, 1);
  }, [planItemDefinition, topElement?.current]);

  const onClickRemove = async () => {
    if (!planItemDefinition) {
      return;
    }
    const { id } = planItemDefinition;
    if (!id) {
      return;
    }
    if (!confirm('Are you sure you want to remove this plan item?')) {
      return;
    }
    removePlanItem(id);
    clearSelectedPlan();
  };

  const onClickExit = async () => {
    clearSelectedPlan();
  };

  const onSubmit = async (data: any) => {
    const { id, description, priority, expression, scaling_components } = data;
    if (!planItemDefinition || !planItemDefinition.ui) {
      return;
    }
    const filteredScalingComponents = scaling_components.filter(
      (scalingComponent: any) => scalingComponent.component_id
    );
    const newPlanItemDefinition: PlanItemDefinition = {
      ...planItemDefinition,
      id,
      description,
      priority,
      expression,
      // scaling_components: [],
      scaling_components: filteredScalingComponents,
    };
    updatePlanItem(newPlanItemDefinition);

    alert('updated!');
  };

  return (
    <div className="plan-drawer drawer drawer-end flex w-full min-w-[48rem] max-w-[48rem] flex-col">
      <input id="drawer" type="checkbox" className="drawer-toggle" checked />
      <div className="drawer-side h-full border-l border-t border-gray-200">
        <div
          className="drawer-content flex h-full w-full flex-col overflow-y-auto"
          ref={topElement}
        >
          <form className="" onSubmit={handleSubmit(onSubmit)}>
            <div className="flex h-14 min-h-14 w-full min-w-full flex-row items-center justify-between border-b border-dashed border-gray-400 bg-gray-75">
              <span className="font-Pretendard truncate whitespace-nowrap px-4 text-lg font-semibold text-gray-1000">
                Plan
              </span>
              <div className="flex px-4">
                <button
                  className="mr-1 flex h-8 items-center justify-center rounded-md border border-red-400  bg-red-400 pl-1 pr-1 text-sm text-gray-50"
                  onClick={onClickRemove}
                >
                  <Image
                    src="/assets/icons/delete.svg"
                    alt="delete.svg"
                    priority={true}
                    width={24}
                    height={24}
                    style={{ minWidth: '1.5rem', maxWidth: '1.5rem' }}
                  />
                </button>
                <button
                  className="ml-1 mr-1 flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50"
                  type="submit"
                >
                  SAVE
                </button>
                <button
                  className="ml-1 flex h-8 items-center justify-center rounded-md border border-gray-600 pl-5 pr-5 text-sm text-gray-600"
                  onClick={onClickExit}
                >
                  EXIT
                </button>
              </div>
            </div>

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text px-2">Plan ID</span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <input
                type="text"
                placeholder="Plan ID"
                className="input-bordered input my-2 w-full px-4 text-sm focus:outline-none"
                autoComplete="off"
                autoCapitalize="off"
                autoCorrect="off"
                {...register('id', { required: true })}
              />
            </div>

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text px-2">Description</span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <textarea
                placeholder="Description"
                className="textarea-bordered textarea textarea-sm my-2 w-full px-4 focus:outline-none"
                autoComplete="off"
                autoCapitalize="off"
                autoCorrect="off"
                {...register('description', { required: true })}
              />
            </div>

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text px-2">Priority</span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <input
                type="number"
                placeholder="Priority"
                className="input-bordered input my-2 w-full px-4 text-sm focus:outline-none"
                autoComplete="off"
                autoCapitalize="off"
                autoCorrect="off"
                {...register('priority', { required: true })}
              />
            </div>

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text px-2">Expression</span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <div className="textarea-bordered textarea textarea-sm my-2 w-full px-4 py-4 focus:outline-none">
                <Controller
                  control={control}
                  name="expression"
                  render={({ field: { onChange, value } }) => (
                    <AceEditor
                      mode="javascript"
                      theme="xcode"
                      onChange={onChange}
                      value={value}
                      editorProps={{
                        $blockScrolling: true,
                      }}
                      setOptions={{
                        showLineNumbers: false,
                        // TODO: Autocomplete
                        // https://github.com/ajaxorg/ace/wiki/How-to-enable-Autocomplete-in-the-Ace-editor
                        enableBasicAutocompletion: true,
                        enableLiveAutocompletion: true,
                        enableSnippets: true,
                        showGutter: false,
                      }}
                      style={{
                        width: '100%',
                        height: '100%',
                        minHeight: '200px',
                      }}
                    />
                  )}
                />
              </div>
            </div>

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text px-2">
                  Scaling Components
                </span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>

              <div className="my-2 w-full">
                {fields?.map((field, index) => (
                  <ScalingComponentPlanSelect
                    key={field.id}
                    {...{ control, index, field, remove }}
                  />
                ))}

                <div className="flex w-full">
                  <button
                    className="flex h-8 w-full items-center justify-center rounded-md border border-gray-600 pl-5 pr-5 text-sm text-gray-600"
                    onClick={() => append({})}
                  >
                    ADD SCALING COMPONENTS
                  </button>
                </div>
              </div>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}
