'use client';
import { useParams } from 'next/navigation';
import { Controller, useForm } from 'react-hook-form';
import AceEditor from 'react-ace';
import { PlanItemDefinition } from '@/types/bindings/plan-item-definition';
import { usePlanStore } from '../plan-store';
import { useEffect } from 'react';
import 'ace-builds/src-noconflict/mode-javascript';
import 'ace-builds/src-noconflict/snippets/javascript';
import 'ace-builds/src-noconflict/theme-xcode';
// import 'ace-builds/src-noconflict/ext-language_tools';

export default function PlanItemDrawer({
  planItemDefinition,
}: {
  planItemDefinition?: PlanItemDefinition;
}) {
  const { register, handleSubmit, control, reset, setValue } = useForm();
  const { id: scalingPlanId } = useParams();
  const clearSelectedPlan = usePlanStore((state) => state.clearSelectedPlan);
  const updatePlanItem = usePlanStore((state) => state.updatePlanItem);
  const removePlanItem = usePlanStore((state) => state.removePlanItem);

  useEffect(() => {
    reset();
    if (!planItemDefinition) {
      return;
    }
    const { id, description, priority, expression } = planItemDefinition;
    setValue('id', id);
    setValue('description', description);
    setValue('priority', priority);
    setValue('expression', expression);
  }, [planItemDefinition]);

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
    await removePlanItem(scalingPlanId, id);
    await clearSelectedPlan(scalingPlanId);
  };

  const onSubmit = async (data: any) => {
    const { id, description, priority, expression } = data;
    if (
      !planItemDefinition ||
      !planItemDefinition.ui ||
      !planItemDefinition.scaling_components
    ) {
      return;
    }
    const newPlanItemDefinition: PlanItemDefinition = {
      ...planItemDefinition,
      id,
      description,
      priority,
      expression,
    };

    updatePlanItem(scalingPlanId, newPlanItemDefinition);

    alert('updated!');
  };

  return (
    <div className="plan-drawer drawer drawer-end w-[32rem]">
      <input id="drawer" type="checkbox" className="drawer-toggle" checked />
      <div className="drawer-side w-[32rem] border-l border-base-300">
        <div className="drawer-content overflow-y-auto bg-base-100 p-4">
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
                {/* <Controller
                  control={control}
                  name="expression"
                  render={({ field: { onChange, onBlur, value, ref } }) => (
                    <MentionsInput
                      value={value}
                      onChange={(
                        event,
                        newValue,
                        newPlainTextValue,
                        mentions
                      ) => {
                        console.log('event', event);
                        onChange(newValue);
                      }}
                      // style={defaultStyle}
                      className="textarea-mention"
                      placeholder={"Mention people using '@'"}
                      a11ySuggestionsListLabel={'Suggested mentions'}
                    >
                      <Mention
                        markup="@[__display__](user:__id__)"
                        trigger="@"
                        data={[
                          {
                            id: 'walter',
                            display: 'Walter White',
                          },
                          {
                            id: 'pipilu',
                            display: '皮皮鲁',
                          },
                          {
                            id: 'luxixi',
                            display: '鲁西西',
                          },
                          {
                            id: 'satoshi1',
                            display: '中本聪',
                          },
                          {
                            id: 'satoshi2',
                            display: 'サトシ・ナカモト',
                          },
                          {
                            id: 'nobi',
                            display: '野比のび太',
                          },
                          {
                            id: 'sung',
                            display: '성덕선',
                          },
                          {
                            id: 'jesse',
                            display: 'Jesse Pinkman',
                          },
                          {
                            id: 'gus',
                            display: 'Gustavo "Gus" Fring',
                          },
                          {
                            id: 'saul',
                            display: 'Saul Goodman',
                          },
                          {
                            id: 'hank',
                            display: 'Hank Schrader',
                          },
                          {
                            id: 'skyler',
                            display: 'Skyler White',
                          },
                          {
                            id: 'mike',
                            display: 'Mike Ehrmantraut',
                          },
                          {
                            id: 'lydia',
                            display: 'Lydìã Rôdarté-Qüayle',
                          },
                        ]}
                        renderSuggestion={(
                          suggestion,
                          search,
                          highlightedDisplay,
                          index,
                          focused
                        ) => (
                          <div className={`user ${focused ? 'focused' : ''}`}>
                            {highlightedDisplay}
                          </div>
                        )}
                        // onAdd={onAdd}
                        // style={defaultMentionStyle}
                      />
                    </MentionsInput>
                  )}
                /> */}
              </div>
            </div>
            {/* {metadataFormControls} */}
          </form>
        </div>
      </div>
    </div>
  );
}
