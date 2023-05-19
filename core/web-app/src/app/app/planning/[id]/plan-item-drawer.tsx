'use client';
import { useParams } from 'next/navigation';
import { Controller, useFieldArray, useForm } from 'react-hook-form';
import AceEditor from 'react-ace';
import { PlanItemDefinition } from '@/types/bindings/plan-item-definition';
import { usePlanStore } from '../plan-store';
import { useEffect, useRef } from 'react';
import 'ace-builds/src-noconflict/mode-javascript';
import 'ace-builds/src-noconflict/snippets/javascript';
import 'ace-builds/src-noconflict/theme-xcode';
import ScalingComponentPlanSelect from './scaling-component-plan-select';
import { omit } from 'lodash';
// import 'ace-builds/src-noconflict/ext-language_tools';

export default function PlanItemDrawer({
  planItemDefinition,
}: {
  planItemDefinition?: PlanItemDefinition;
}) {
  // react-hook-form
  const { register, handleSubmit, control, reset, setValue } = useForm();
  // For Scaling Component List
  const { fields, append, remove } = useFieldArray({
    control,
    name: 'scaling_components',
  });
  //
  const topElement = useRef<HTMLDivElement>(null);
  // Plan data management
  const { id: scalingPlanId } = useParams();
  const clearSelectedPlan = usePlanStore((state) => state.clearSelection);
  const updatePlanItem = usePlanStore((state) => state.updatePlanItem);
  const removePlanItem = usePlanStore((state) => state.removePlanItem);

  // Initialize form data
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

  //
  // Events
  //

  // Remove plan item
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

  // Update plan item
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
    <div className="plan-drawer drawer drawer-end w-[32rem]">
      <input id="drawer" type="checkbox" className="drawer-toggle" checked />
      <div className="drawer-side w-[32rem] border-l border-base-300">
        <div
          className="drawer-content overflow-y-auto bg-base-100 p-4"
          ref={topElement}
        >
          <form className="" onSubmit={handleSubmit(onSubmit)}>
            <div className="mb-4 flex items-center justify-between">
              <h2 className="font-bold">Plan</h2>
              <div>
                <button
                  type="button"
                  className="btn-error btn-sm btn mr-2"
                  onClick={onClickRemove}
                >
                  Remove
                </button>
                <button type="submit" className="btn-primary btn-sm btn">
                  Save
                </button>
              </div>
            </div>
            <div className="form-control mb-4 w-full">
              <label className="label">
                <span className="label-text">Plan ID</span>
                {/* <span className="label-text-alt">used as a variable name</span> */}
              </label>
              <input
                type="text"
                placeholder="Type here"
                className="input-bordered input input-md w-full"
                autoComplete="off"
                autoCapitalize="off"
                autoCorrect="off"
                {...register('id', { required: true })}
              />
            </div>
            <div className="form-control mb-4 w-full">
              <label className="label">
                <span className="label-text">Description</span>
                {/* <span className="label-text-alt">used as a variable name</span> */}
              </label>
              <textarea
                placeholder="Type here"
                className="textarea-bordered textarea textarea-md w-full"
                {...register('description', { required: false })}
              />
            </div>
            <div className="form-control mb-4 w-full">
              <label className="label">
                <span className="label-text">Priority</span>
                {/* <span className="label-text-alt">used as a variable name</span> */}
              </label>
              <input
                type="number"
                placeholder="Type here"
                className="input-bordered input input-md w-full"
                {...register('priority', { required: false })}
              />
            </div>
            <div className="form-control mb-4 w-full">
              <label className="label">
                <span className="label-text">Expression</span>
                {/* <span className="label-text-alt">
                  Use '@' to see the metric variables
                </span> */}
              </label>
              <div className="textarea-bordered textarea textarea-md w-full">
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
            <div className="form-control mb-4 w-full">
              <label className="label">
                <span className="label-text">Scaling Components</span>
                {/* <span className="label-text-alt">used as a variable name</span> */}
              </label>
              {/* Scaling Component List */}
              <div className="">
                {fields?.map((field, index) => (
                  <ScalingComponentPlanSelect
                    key={field.id}
                    {...{ control, index, field, remove }}
                  />
                ))}
                <div className="flex justify-end">
                  <button
                    type="button"
                    className="btn-primary btn-outline btn-xs btn w-full"
                    onClick={() => append({})}
                  >
                    Add Scaling Components
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
