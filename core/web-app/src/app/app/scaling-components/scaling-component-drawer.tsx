import React, { useEffect } from 'react';
import { Controller, useForm } from 'react-hook-form';
import Image from 'next/image';
import ScalingComponentService from '@/services/scaling-component';
import { generateScalingComponentDefinition } from '@/utils/scaling-component-binding';
import yaml from 'js-yaml';
import { ScalingComponentDefinitionEx } from './scaling-component-definition-ex';
import { transformDefinitionId } from '@/utils/definition-id';
import YAMLEditor from '../common/yaml-editor';
import { set } from 'lodash';

export default function ScalingComponentDetailDrawer({
  scalingComponent,
  onClose,
  onChange,
}: {
  scalingComponent?: ScalingComponentDefinitionEx;
  onClose: () => void;
  onChange: (fetchFlag: boolean) => void;
}) {
  const {
    formState: { isValid },
    register,
    handleSubmit,
    control,
    setValue,
    reset,
  } = useForm();

  const dbId = scalingComponent?.db_id;
  const isNew = !dbId;

  useEffect(() => {
    if (isNew) {
      reset();
      return;
    }

    const { kind, db_id, id, component_kind, metadata, enabled, ...rest } =
      scalingComponent;
    setValue('kind', kind);
    setValue('db_id', db_id);
    setValue('id', id);
    setValue('component_kind', component_kind);
    setValue('enabled', enabled);
    let metadataYaml = '';
    if (Object.keys(metadata).length > 0) {
      metadataYaml = yaml.dump(metadata);
    }
    setValue('metadata', metadataYaml);
  }, [scalingComponent]);

  const onClickInitialize = async () => {
    reset();
  };

  const onClickRemove = async () => {
    if (isNew) {
      return;
    }

    if (!confirm('Are you sure to delete this scaling component?')) {
      return;
    }

    try {
      const response = await ScalingComponentService.deleteScalingComponent(
        dbId
      );
      console.info({ response });

      onChange(true);
      onClose();
    } catch (error) {
      console.error(error);
    }
  };

  const onSubmit = async (data: any) => {
    const { kind, db_id, id, component_kind, metadata, enabled, ...rest } =
      data;

    const scalingComponent = generateScalingComponentDefinition({
      kind: 'ScalingComponent',
      id: id,
      db_id: dbId,
      component_kind: component_kind,
      enabled: enabled,
      metadata: yaml.load(metadata ?? ''),
    });
    try {
      if (isNew) {
        const response = await ScalingComponentService.createScalingComponent(
          scalingComponent
        );
      } else {
        const response = await ScalingComponentService.updateScalingComponent(
          scalingComponent
        );
        console.info({ response });
      }
      onChange(true);
      onClose();
    } catch (error) {
      console.error(error);
    }
  };

  return (
    <div className="scaling-component-drawer drawer drawer-end fixed bottom-0 right-0 top-16 z-20 w-full">
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
          onClick={() => onClose()}
        />
        <div className="drawer-content flex h-full min-w-[48rem] max-w-[48rem] flex-col overflow-y-auto border-l border-gray-200 bg-base-100 pb-20">
          <form className="" onSubmit={handleSubmit(onSubmit)}>
            <div className="flex h-14 min-h-14 w-full min-w-full flex-row items-center justify-between border-b border-dashed border-gray-400 bg-gray-75">
              <span className="font-Pretendard truncate whitespace-nowrap px-4 text-lg font-semibold text-gray-1000">
                Scaling Component
              </span>
              <div className="flex px-4">
                {isNew ? (
                  <button
                    className="ml-1 mr-1 flex h-8 items-center justify-center rounded-md border border-red-400 bg-red-400 pl-5 pr-5 text-sm text-gray-50"
                    onClick={onClickInitialize}
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
                  className="ml-1 mr-1 flex h-8 items-center justify-center rounded-md border border-blue-400 bg-blue-400 pl-5 pr-5 text-sm text-gray-50 disabled:border-gray-400 disabled:bg-gray-400"
                  type="submit"
                  disabled={!isValid}
                >
                  SAVE
                </button>
                <button
                  className="ml-1 flex h-8 items-center justify-center rounded-md border border-gray-600 pl-5 pr-5 text-sm text-gray-600"
                  onClick={() => onClose()}
                  type="button"
                >
                  EXIT
                </button>
              </div>
            </div>

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text">Scaling Component ID</span>
              </label>
              <Controller
                control={control}
                name="id"
                rules={{ required: true }}
                render={({ field: { onChange, value } }) => (
                  <input
                    type="text"
                    placeholder="Scaling Component ID"
                    className="input-bordered input my-2 w-full px-4 text-sm focus:outline-none"
                    autoComplete="off"
                    autoCapitalize="off"
                    autoCorrect="off"
                    value={value ?? ''}
                    onChange={(event) => {
                      event.target.value = transformDefinitionId(
                        event.target.value
                      );
                      onChange(event);
                    }}
                  />
                )}
              />
            </div>

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text">
                  Scaling Component Kind
                </span>
              </label>
              {/* TODO: input => select */}
              <input
                type="text"
                placeholder="Scaling Component Kind"
                className="input-bordered input my-2 w-full px-4 text-sm focus:outline-none"
                autoComplete="off"
                autoCapitalize="off"
                autoCorrect="off"
                {...register('component_kind', { required: true })}
              />
            </div>
            {/* Enabled */}
            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text">Enabled</span>
              </label>
              <input
                type="checkbox"
                className="checkbox"
                {...register('enabled', {})}
              />
            </div>
            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text">
                  Scaling Component Metadata
                </span>
              </label>
              <div className="textarea-bordered textarea textarea-sm my-2 w-full px-4 py-4 focus:outline-none">
                <Controller
                  control={control}
                  name="metadata"
                  render={({ field: { onChange, value } }) => (
                    <YAMLEditor value={value} onChange={onChange} />
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
