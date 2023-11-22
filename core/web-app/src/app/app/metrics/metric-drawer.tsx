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

import MetricService from '@/services/metric';
import { generateMetricDefinition } from '@/utils/metric-binding';
import { MetricDefinition } from '@/types/bindings/metric-definition';

export default function MetricDetailDrawer({
  metricsItem,
  setDetailsModalFlag,
  setFetchFlag,
}: {
  metricsItem?: MetricDefinition;
  setDetailsModalFlag: (detailsModalFlag: boolean) => void;
  setFetchFlag: (fetchFlag: boolean) => void;
}) {
  const { register, handleSubmit, control, setValue, reset } = useForm();

  const dbId = metricsItem?.db_id;
  const isNew = !dbId;

  const yaml = require('js-yaml');

  useEffect(() => {
    if (isNew) {
      return;
    }

    const { kind, db_id, id, collector, metadata, ...rest } = metricsItem;
    setValue('kind', kind);
    setValue('db_id', db_id);
    setValue('id', id);
    setValue('collector', collector);
    setValue('metadata', yaml.dump(metadata));
  }, [metricsItem]);

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
      const response = await MetricService.deleteMetric(dbId);
      console.info({ response });

      setFetchFlag(true);
      setDetailsModalFlag(false);
    } catch (error) {
      console.error(error);
    }
  };

  const onSubmit = async (data: any) => {
    const { kind, db_id, id, collector, metadata, ...rest } = data;

    const metricsItem = generateMetricDefinition({
      kind: 'Metric',
      id: id,
      db_id: dbId,
      collector: collector,
      metadata: yaml.load(metadata),
    });
    try {
      if (isNew) {
        const response = await MetricService.createMetric(metricsItem);
        console.info({ response, isNew });
      } else {
        const response = await MetricService.updateMetric(metricsItem);
        console.info({ response });
      }
      setFetchFlag(true);
      setDetailsModalFlag(false);
    } catch (error) {
      console.error(error);
    }
  };

  return (
    <div className="metric-drawer drawer drawer-end fixed bottom-0 right-0 top-16 z-50 w-full">
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
                Metric
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
                <span className="text-md label-text px-2">Metric ID</span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <input
                type="text"
                placeholder="Metric ID"
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
                  Metric Collector
                </span>
                {/* <span className="label-text-alt">label-text-alt</span> */}
              </label>
              <input
                type="text"
                placeholder="Metric Collector"
                className="input-bordered input my-2 w-full px-4 text-sm focus:outline-none"
                autoComplete="off"
                autoCapitalize="off"
                autoCorrect="off"
                {...register('collector', { required: true })}
              />
            </div>

            <div className="form-control w-full px-4 py-2">
              <label className="label px-0 py-2">
                <span className="text-md label-text px-2">Metric Metadata</span>
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