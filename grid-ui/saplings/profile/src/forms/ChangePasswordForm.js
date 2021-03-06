/**
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
import React, { useState, useEffect } from 'react';
import { sha256 } from 'js-sha256';
import { getSharedConfig } from 'splinter-saplingjs';
import { MultiStepForm, Step, StepInput } from './MultiStepForm';
import { http } from '../http';
import { Loader } from '../Loader';
import { useDebounce } from '../useDebounce';

export function ChangePasswordForm() {
  const [state, setState] = useState({
    password: null,
    newPassword: null,
    confirmPassword: null
  });

  const [loading, setLoading] = useState(false);
  const [loadingState, setLoadingState] = useState(null);
  const [error, setError] = useState(null);

  const debouncedNewPassword = useDebounce(state.newPassword, 500);
  const debouncedConfirmPassword = useDebounce(state.confirmPassword, 500);

  useEffect(() => {
    if (debouncedNewPassword && debouncedConfirmPassword) {
      if (debouncedNewPassword !== debouncedConfirmPassword) {
        setError('Passwords do not match');
      } else {
        setError(null);
      }
    }
  }, [debouncedNewPassword, debouncedConfirmPassword]);

  useEffect(() => {
    setTimeout(() => {
      setLoadingState(null);
      setLoading(false);
    }, 5000);
  }, [loadingState]);

  const submitChangePassword = async () => {
    setLoading(true);

    const { displayName, token, userId } = JSON.parse(
      window.sessionStorage.getItem('CANOPY_USER')
    );

    const hashedPassword = sha256.hmac
      .create(displayName)
      .update(state.password)
      .hex();

    const newHashedPassword = sha256.hmac
      .create(displayName)
      .update(debouncedNewPassword)
      .hex();

    const body = JSON.stringify({
      username: displayName,
      hashed_password: hashedPassword,
      new_password: newHashedPassword
    });

    try {
      const { splinterURL } = getSharedConfig().canopyConfig;
      await http(
        'PUT',
        `${splinterURL}/biome/users/${userId}`,
        body,
        request => {
          request.setRequestHeader('Authorization', `Bearer ${token}`);
        }
      );
      setLoadingState('success');
      setState({
        password: null,
        newPassword: null,
        confirmPassword: null
      });
    } catch (err) {
      const e = JSON.parse(err);
      setLoadingState('failure');
      setError(e.message);
    }
  };

  const handleChange = event => {
    const { name, value } = event.target;
    setState({
      ...state,
      [name]: value
    });
  };

  return (
    <div
      className="wrapper form-wrapper"
      style={{ width: '50vw', 'min-height': '50vh' }}
    >
      {!loading && (
        <MultiStepForm
          formName="Change password"
          handleSubmit={submitChangePassword}
          disabled={
            !(
              state.password &&
              debouncedNewPassword &&
              debouncedConfirmPassword
            )
          }
          error={error}
        >
          <Step step={1}>
            <StepInput
              type="password"
              label="Current password"
              id="existing-password"
              name="password"
              value={state.password}
              onChange={handleChange}
              required
            />
          </Step>
          <Step step={2}>
            <StepInput
              type="password"
              label="New password"
              id="new-password"
              name="newPassword"
              value={state.newPassword}
              onChange={handleChange}
              required
            />
            <StepInput
              type="password"
              label="Confirm password"
              id="confirm-password"
              name="confirmPassword"
              value={state.confirmPassword}
              onChange={handleChange}
              required
            />
          </Step>
        </MultiStepForm>
      )}
      {loading && <Loader state={loadingState} />}
      {error && (
        <div className="error" style={{ color: 'var(--color-failure' }}>
          <span>{error}</span>
        </div>
      )}
    </div>
  );
}
