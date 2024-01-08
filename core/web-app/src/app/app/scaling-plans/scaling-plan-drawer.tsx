import React, { useEffect } from 'react';
import { Controller, useForm } from 'react-hook-form';
import Image from 'next/image';
import { produce } from 'immer';
import ScalingPlanService from '@/services/scaling-plan';
import { generateScalingPlanDefinition } from '@/utils/scaling-plan-binding';
import { ScalingPlanDefinitionEx } from './scaling-plan-definition-ex';
import yaml from 'js-yaml';
import {
  DEFINITION_ID_RULE_DESCRIPTION,
  transformDefinitionId,
} from '@/utils/definition-id';
import dynamic from 'next/dynamic';

const YAMLEditor = dynamic(() => import('../common/yaml-editor'), {
  ssr: false,
});

export default function ScalingPlanDrawer({
  scalingPlan: scalingPlan,
  onClose,
  onChange,
}: {
  scalingPlan?: ScalingPlanDefinitionEx | undefined;
  // setScalingPlansItem: (
  //   scalingPlan: ScalingPlanDefinitionEx | undefined
  // ) => void;
  // setDetailsModalFlag: (detailsModalFlag: boolean) => void;
  onClose: () => void;
  onChange: () => void;
}) {
  const { register, handleSubmit, control, setValue, getValues, reset } =
    useForm();

  const isNew = !scalingPlan?.db_id;

  useEffect(() => {
    if (isNew || !scalingPlan) {
      return;
    }

    const { kind, db_id, id, metadata, plans, enabled, ...rest } = scalingPlan;
    setValue('kind', kind);
    setValue('db_id', db_id);
    setValue('id', id);
    setValue('metadata', yaml.dump(metadata));
    setValue('enabled', enabled);

    const newScalingPlanDefinition: ScalingPlanDefinitionEx = {
      kind,
      db_id,
      id,
      metadata,
      plans,
      enabled,
    };
    const scalingPlanForDiagram = produce(
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
    setValue('plans', yaml.dump(scalingPlanForDiagram.plans));
  }, [scalingPlan, isNew]);

  const onClickOverlay = () => {
    /* TODO */
    /* Possible triggers for data synchronization. */
    onClose();
  };

  const onClickExit = async () => {
    /* TODO */
    /* Possible triggers for data synchronization. */
    onClose();
  };

  const onClickReset = async () => {
    reset();
  };

  const onClickRemove = async () => {
    const dbId = scalingPlan?.db_id;
    if (!dbId) {
      return;
    }

    if (isNew) {
      onClose();
      return;
    }

    try {
      const response = await ScalingPlanService.deleteScalingPlan(dbId);
      // TODO: Check response
      onChange();
      onClose();
    } catch (error) {
      console.error(error);
    }
  };

  const onSubmit = async (data: any) => {
    const { kind, db_id, id, metadata, plans, ...rest } = data;
    const newScalingPlan = generateScalingPlanDefinition({
      kind: 'ScalingPlan',
      id: id,
      db_id: isNew ? '' : db_id,
      metadata: yaml.load(metadata ?? ''),
      plans: yaml.load(plans ?? '') as any[],
    });
    try {
      if (isNew) {
        const result = await ScalingPlanService.createScalingPlan(
          newScalingPlan
        );
        console.info({ result, isNew });
      } else {
        const result = await ScalingPlanService.updateScalingPlan(
          newScalingPlan
        );
        console.info({ result });
      }
      onChange();
      onClose();
    } catch (error) {
      console.error(error);
    }
  };

  return (
    <div className="scaling-plan-drawer drawer drawer-end fixed bottom-0 right-0 top-16 z-20 w-full">
      <input
        id="drawer"
        type="checkbox"
        className="drawer-toggle"
        defaultChecked={true}
      />
      <div className="drawer-side h-full border-t border-gray-200">
        <label
          htmlFor="drawer"
          className="drawer-overlay"
          onClick={onClickOverlay}
        />
        <div className="drawer-content flex h-full min-w-[48rem] max-w-[48rem] flex-col overflow-y-auto border-l border-gray-200 bg-base-100 pb-20">
          <form onSubmit={handleSubmit(onSubmit)}>
            <div className="flex h-14 min-h-14 w-full min-w-full flex-row items-center justify-between border-b border-dashed border-gray-400 bg-gray-75">
              <span className="font-Pretendard truncate whitespace-nowrap px-4 text-lg font-semibold text-gray-1000">
                Scaling Plan
              </span>
              <div className="flex px-4">
                {isNew ? (
                  <button
                    className="ml-1 mr-1 flex h-8 items-center justify-center rounded-md border border-red-400 bg-red-400 pl-5 pr-5 text-sm text-gray-50"
                    onClick={onClickReset}
                    type="button"
                  >
                    RESET
                  </button>
                ) : (
                  <button
                    className="mr-1 flex h-8 items-center justify-center rounded-md border border-red-400  bg-red-400 pl-1 pr-1 text-sm text-gray-50"
                    onClick={onClickRemove}
                    type="button"
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
                )}
                <button
                  className="ml-1 mr-1 flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50"
                  type="submit"
                >
                  SAVE
                </button>
                <button
                  className="ml-1 flex h-8 items-center justify-center rounded-md border border-gray-600 pl-5 pr-5 text-sm text-gray-600"
                  onClick={onClickExit}
                  type="button"
                >
                  EXIT
                </button>
              </div>
            </div>
            {/* Scaling Plan ID */}
            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text">
                  Scaling Plan ID {DEFINITION_ID_RULE_DESCRIPTION}
                </span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>

              <Controller
                control={control}
                name="id"
                render={({ field: { onChange, value } }) => (
                  <input
                    type="text"
                    placeholder="Scaling Plan ID"
                    className="input-bordered input my-2 w-full px-4 text-sm focus:outline-none"
                    autoComplete="off"
                    autoCapitalize="off"
                    autoCorrect="off"
                    onChange={(event) => {
                      // let value = event.target.value;
                      event.target.value = transformDefinitionId(
                        event.target.value
                      );
                      onChange(event);
                    }}
                  />
                )}
              />
            </div>
            {/* Enabled */}
            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text">Enabled</span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <input
                type="checkbox"
                className="checkbox"
                {...register('enabled')}
              />
            </div>
            {/* Metadata */}
            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text">Metadata (YAML)</span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <div className="textarea-bordered textarea textarea-sm my-2 w-full px-4 py-4 focus:outline-none">
                <Controller
                  control={control}
                  name="metadata"
                  render={({ field: { onChange, value } }) => (
                    <YAMLEditor onChange={onChange} value={value} />
                  )}
                />
              </div>
            </div>
            {/* Plans */}
            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text">Plans (YAML)</span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <div className="textarea-bordered textarea textarea-sm my-2 w-full px-4 py-4 focus:outline-none">
                <Controller
                  control={control}
                  name="plans"
                  render={({ field: { onChange, value } }) => (
                    <YAMLEditor onChange={onChange} value={value} />
                  )}
                />
              </div>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}
