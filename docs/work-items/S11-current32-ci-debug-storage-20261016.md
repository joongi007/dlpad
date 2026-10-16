# R11 current32 CI 저장비용 조절 후보 v2

- 상태: proposed / Root 승인범위 후보준비 완료, 실제실행 대기
- 분류: validation
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: p0_audit / 새 B후보파일만. Live/workflow/docsfreeze/원v1prepare/source/runtime/cache/3PE/Cargo/lock 변경0.
- 선행 작업: actualwhole32/rootreview,원v1prepareterminal0,Root실제disk확인·모든Cargo/native/GUI terminal

## 사용자 결과

테스트·실행검사기준을줄이지않고dev/testdebugsymbols만줄여독립CI실행의디스크부담을줄일후보를준비한다. 절감량은미측정이다.

## 계약과 범위

원v24target8.016GiB/debugPDB4.070GiB를실제읽었다. DEV_DEBUG=0/TEST_DEBUG=0만설정하고incremental/optimization/debugassertions/overflow/panic/release/jobs는유지한다. 원15/8/7/actualRoot rawharness합계를검사. 원v1export/source/cache/supply읽기재사용+새runtime38/_pth1경로수정+새config/output. 원userGit/원prepared결과덮기0.

## 인수 조건

runtime37byte그대로/_pth정확기존1경로만교체,원38namespace검사. configoutput/pythonpath외변경0. actualchildenv2keys0 및다른CARGO_PROFILE_*거절,원15step/envreceipt/최종profile해시/독립review검사. 원3PE/source/supply성공·실패보존,새fresh-source target만사용. 디스크실제여유재확인후Root실행;StorageFull이나실패를숨기지않음.

## 검증과 handoff

PythonAST/11pure·mock테스트PASS,실제prepare/Cargo/CI/native/Git0. Rootargv는argv.json. 원CI상한7200/원7capture상한유지. runtime새24MiB사본·새source1복사/cache88.7MB/새target별도. debug0의단계별/peak/실제절감은실행후검토. Liveworkflow는Root가freeze해제후제안patch별도통합;현재는변경0.

Root 결정: 기존실측PDB4.070GiB를근거로 dev/test debug정보0만 채택해 새localCI를실행한다. 원workflow는별도미적용이며hostedCI인수로표현하지않는다. 원v1prepared export/runtime/cache/원Git/675소스/default3PE를보존하고새config/runtime/output만쓴다. 실제절감은결과로측정하며가정으로PASS하지않는다. 원38cache읽기대조PASS,추가76삭제0.


## 2026-10-16 실제 종료와 다음 조건

현재 lowdebug v2 실행은 cpu-release의 Cargo101/OS112 저장 공간 부족으로 실패했다. 앞선 fmt/clippy/test/release 성공은 전체15단계 성공을 뜻하지 않는다. 원 로그와 failure 결과는 보존했고 정확한 이번 소유 실패 target만 제거해 여유5,207,900,160B를 확보했다. lowstorage v3(debug=0/incremental=0)는 준비만 했고 NOT RUN이다. 이후 README/CONTEXT 등 작성 문서를 갱신했으므로 재실행 전에 원 supply 검사를 완화하지 않고 새 export/supply snapshot을 준비해야 한다. hosted CI나 Git remote는 연결하지 않았다.
