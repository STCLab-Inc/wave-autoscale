'use client';

import React, { useEffect, useMemo, useState } from 'react';
import { useRouter } from 'next/navigation';
import { useForm } from 'react-hook-form';
import { forEach } from 'lodash';
import Image from 'next/image';

import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';
import {
  generateScalingComponentDefinition,
  getScalingComponentKeyTypes,
} from '@/utils/scaling-component-binding';
import ScalingComponentService from '@/services/scaling-component';
import { getMetadataFormControls } from './metadata-form-controls';

const componentKeyTypes = getScalingComponentKeyTypes();
const componentOptions = componentKeyTypes.map((componentKeyType) => (
  <option
    key={componentKeyType.componentName}
    value={componentKeyType.componentName}
  >
    {componentKeyType.componentName}
  </option>
));

export default function ScalingComponentDetailDrawer({
  componentDefinition,
}: {
  componentDefinition?: ScalingComponentDefinition;
}) {
  const { register, handleSubmit, setValue, getValues, reset } = useForm();
  const router = useRouter();
  const dbId = componentDefinition?.db_id;
  const isNew = !dbId;
  const [selectedComponentKind, setSelectedComponentKind] = useState<string>();

  const metadataFormControls = useMemo(() => {
    if (selectedComponentKind) {
      const keyTypes = componentKeyTypes.find(
        (componentKeyType) =>
          componentKeyType.componentName === selectedComponentKind
      )?.keyTypes;
      if (keyTypes) {
        return getMetadataFormControls(keyTypes, register);
      }
    }
  }, [selectedComponentKind]);

  const goBack = (refresh?: boolean) => {
    let path = window.location.href;
    path = path.slice(0, path.lastIndexOf('/'));
    router.push(path);
    if (refresh) {
      router.refresh();
    }
  };

  useEffect(() => {
    if (isNew) {
      return;
    }
    const { id, component_kind, metadata, ...rest } = componentDefinition;
    setValue('id', id);
    setValue('component_kind', component_kind);
    forEach(metadata, (value, key) => {
      setValue(key, value);
    });
    setSelectedComponentKind(component_kind);
  }, [componentDefinition, isNew]);

  const onClickOverlay = () => {
    goBack(false);
  };

  const onChangeComponentKind = (e: any) => {
    const componentKind = e.target.value;
    setSelectedComponentKind(componentKind);

    const id = getValues('id');
    reset();
    setValue('id', id);
    setValue('component_kind', componentKind);
  };

  const onClickExit = async () => {
    goBack(false);
  };

  const onClickInitialize = async () => {
    setSelectedComponentKind('Scaling Component Kind');

    const id = getValues('id');
    reset();
    setValue('id', id);
    setValue('component_kind', 'Scaling Component Kind');
  };

  const onClickRemove = async () => {
    if (isNew || !dbId) {
      return;
    }
    try {
      await ScalingComponentService.deleteScalingComponent(dbId);
      goBack(true);
    } catch (error) {
      console.log(error);
    }
  };

  const onSubmit = async (data: any) => {
    const { id, component_kind, ...metadata } = data;
    const componentDefinition = generateScalingComponentDefinition({
      id,
      db_id: isNew ? '' : dbId,
      component_kind,
      metadata,
    });
    try {
      if (isNew) {
        const result = await ScalingComponentService.createScalingComponent(
          componentDefinition
        );
        console.log({ result, isNew });
      } else {
        const result = await ScalingComponentService.updateScalingComponent(
          componentDefinition
        );
        console.log({ result });
      }
      goBack(true);
    } catch (error) {
      console.log(error);
    }
  };

  return (
    <div className="scaling-components-drawer drawer drawer-end fixed bottom-0 right-0 top-16 z-50 w-full border-t border-gray-200">
      <input id="drawer" type="checkbox" className="drawer-toggle" checked />
      <div className="drawer-side h-full border-t border-gray-200">
        <label
          htmlFor="drawer"
          className="drawer-overlay"
          onClick={onClickOverlay}
        />
        <div className="drawer-content flex h-full min-w-[25rem] max-w-[25rem] flex-col overflow-y-auto border-l border-gray-200 bg-base-100 pb-20">
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

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text px-2">
                  Scaling Component Type
                </span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <select
                className="select-bordered select my-2 flex w-full truncate rounded-md text-sm focus:outline-none"
                defaultValue="Scaling Component Kind"
                {...register('component_kind', {
                  required: true,
                  onChange: onChangeComponentKind,
                })}
              >
                <option disabled>Scaling Component Kind</option>
                {componentOptions}
              </select>
            </div>

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text px-2">
                  Scaling Component ID
                </span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <input
                type="text"
                placeholder="Scaling Component ID"
                className="input-bordered input my-2 w-full px-4 text-sm focus:outline-none"
                autoComplete="off"
                autoCapitalize="off"
                autoCorrect="off"
                {...register('id', { required: true })}
              />
            </div>

            {metadataFormControls}
          </form>
        </div>
      </div>
    </div>
  );
}
