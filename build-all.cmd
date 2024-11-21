@echo off

cd %~dp0

call build-manager.cmd

call build-worker.cmd
