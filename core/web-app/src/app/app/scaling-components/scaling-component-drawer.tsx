'use client';

import React, { useEffect } from 'react';

import { Controller, useForm } from 'react-hook-form';
import Image from 'next/image';
// import 'ace-builds/src-noconflict/ext-language_tools';
import AceEditor from 'react-ace';
import 'ace-builds/src-noconflict/mode-javascript';
import 'ace-builds/src-noconflict/snippets/javascript';
import 'ace-builds/src-noconflict/theme-xcode';
import 'ace-builds/src-noconflict/mode-yaml';

import ScalingComponentService from '@/services/scaling-component';
import { ScalingComponentDefinition } from '@/types/bindings/scaling-component-definition';
import { generateScalingComponentDefinition } from '@/utils/scaling-component-binding';

export default function ScalingComponentDetailDrawer({
  scalingComponentsItem,
  setDetailsModalFlag,
  setFetchFlag,
}: {
  scalingComponentsItem?: ScalingComponentDefinition;
  setDetailsModalFlag: (detailsModalFlag: boolean) => void;
  setFetchFlag: (fetchFlag: boolean) => void;
}) {
  const { register, handleSubmit, control, setValue, reset } = useForm();

  const dbId = scalingComponentsItem?.db_id;
  const isNew = !dbId;

  const yaml = require('js-yaml');

  useEffect(() => {
    if (isNew) {
      return;
    }

    const { kind, db_id, id, component_kind, metadata, ...rest } =
      scalingComponentsItem;
    setValue('kind', kind);
    setValue('db_id', db_id);
    setValue('id', id);
    setValue('component_kind', component_kind);
    setValue('metadata', yaml.dump(metadata));
  }, [scalingComponentsItem]);

  const onClickOverlay = () => {
    setFetchFlag(true);
    setDetailsModalFlag(false);
  };

  const onClickExit = async () => {
    setFetchFlag(true);
    setDetailsModalFlag(false);
  };

  const onClickInitialize = async () => {
    reset();
  };

  const onClickRemove = async () => {
    if (isNew) {
      return;
    }
    try {
      const response = await ScalingComponentService.deleteScalingComponent(
        dbId
      );
      console.info({ response });

      setFetchFlag(true);
      setDetailsModalFlag(false);
    } catch (error) {
      console.error(error);
    }
  };

  const onSubmit = async (data: any) => {
    const { kind, db_id, id, component_kind, metadata, ...rest } = data;

    const scalingComponentsItem = generateScalingComponentDefinition({
      kind: 'ScalingComponent',
      id: id,
      db_id: dbId,
      component_kind: component_kind,
      metadata: yaml.load(metadata),
    });
    try {
      if (isNew) {
        const response = await ScalingComponentService.createScalingComponent(
          scalingComponentsItem
        );
        console.info({ response, isNew });
      } else {
        const response = await ScalingComponentService.updateScalingComponent(
          scalingComponentsItem
        );
        console.info({ response });
      }
      setFetchFlag(true);
      setDetailsModalFlag(false);
    } catch (error) {
      console.error(error);
    }
  };

  return (
    <div className="scaling-component-drawer drawer drawer-end fixed bottom-0 right-0 top-16 z-50 w-full">
      <input id="drawer" type="checkbox" className="drawer-toggle" checked />
      <div className="drawer-side h-full border-t border-gray-200">
        <label
          htmlFor="drawer"
          className="drawer-overlay"
          onClick={onClickOverlay}
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

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text px-2">
                  Scaling Component Kind
                </span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
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

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text px-2">
                  Scaling Component Metadata
                </span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <div className="textarea-bordered textarea textarea-sm my-2 w-full px-4 py-4 focus:outline-none">
                <Controller
                  control={control}
                  name="metadata"
                  render={({ field: { onChange, value } }) => (
                    <AceEditor
                      mode="yaml"
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
                        minHeight: '40rem',
                      }}
                    />
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
