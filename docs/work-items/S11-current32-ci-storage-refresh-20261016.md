# S11: current32 로컬 CI 저장공간·현재 문서 snapshot 갱신

- 상태: done-scoped — current33 실제 로컬 CI15단계 및 Root 독립검토 통과
- 분류: validation
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: p0_audit; 이 문서와 `.local-tools/session-recovery-20261013/current32-ci-storage-refresh-audit-20261016`, `current32-local-ci-lowstorage-v4`만
- 선행 작업: Root current whole/독립 review·실제 current compile source·기본 3PE 보존, 실제 C/D free 원자료 및 Root의 새 C 사용자 TEMP 쓰기 승인 검토

## 사용자 결과

기존 D 공간 부족과 오래된 작성 문서 snapshot을 재사용하는 문제를 분리한다. 현재 작성 문서를 새로 export하는 실제 실행 후보를 준비하고, 전체 CI 저장·빌드·임시 산출물은 새로 소유한 C TEMP로 한정한다.

## 계약과 범위

기존 15단계/8 Cargo/7 capture와 원 source/tool/cache/lock 보존 검증을 유지한다. DEV/TEST DEBUG=0과 CARGO_INCREMENTAL=0만 저장 정책으로 사용하며 optimization·assertions·테스트 범위·release profile·jobs2·7200초를 바꾸지 않는다. 원 `.git` 복구·원 target 삭제·공유 source/schema/lock/workflow 변경은 하지 않는다. 새 source/export/supply는 Root 준비 명령 시 현재 문서를 동결한다. 실제 C 파일 생성은 Root의 승인 검토를 거쳐 실행한다.

## 인수 조건

원 v2 실패와 v3 미실행·출력 경로 충돌을 보존한다. 새 namespace·현재 authored export·동적 compile/test matrix·기본 3PE와 원 불변 입력을 대조한다. C KnownFolder/SID private root·보유 HANDLE·marker·소유 receipt를 확인하고 원 outer Job·실패 보존을 유지한다. 디스크 사전 여유와 실행 중 watermark는 파일시스템 quota가 아님을 명시한다. hosted CI·전체 P0 성공으로 표현하지 않는다.

## 검증과 handoff

새 v4 후보의 Python AST8개와 pure path/worker/원 owned argv 회귀9개가 통과했다. native C# 컴파일·실제 FS/Job 실패 경로는 아직 미검증이다. 이 작업의 실제 Cargo/GUI/native/CI/삭제는 0이다. 원 C free 610,966,925,312B와 D free 4,726,366,208B는 이전 native 읽기 관측이며 실행 직전 재확인한다. 새 v4 HANDOFF/argv.template에 실제 순서와 미검증 사항을 기록했다. old CI3의 v2 evidence 출력 경로 충돌도 원형 보존하고 v4에서 수정했다. Root의 최종 actual whole와 작성 문서 freeze 전에는 export를 실행하지 않는다.

최종 독립검토 보정: embedded ._pth에서 candidate sys.path 삽입 후 storage_paths를 import한다(원 순서 in-memory RED / 새 순서 GREEN). native catch는 C 파일 읽기·쓰기를 전혀 하지 않고 Console 실패 JSON을 원 D host 로그로 보존한다. C receipt 복사는 exact expected owner·private_root_created_and_verified=true·native parent/root ID가 모두 있는 원 summary에서만 허용한다. collision/precreate failure는 C receipt read/copy0이며, 생성 후 실패의 partial root는 보존한다. 실제 native/Cargo/prepare는 여전히0이다. whole33을 포함한 최종 실제 caller proof는 명시 인자로 받으며 숫자·hash를 예측하지 않는다.

Linux mutation alias: prepare/run/review 후보 D와 shared observer ROOT/TOOLS를 권한 있는 소문자 `/mnt/d/project/dlpad`에 명시 결합한다. 허용된 lowercase 작업 경로에 명시 결합하며 shared 파일/Windows path guard는 수정하지 않는다. pure 회귀 포함 최종9PASS, AST8PASS.

Root 실행 직전 기준은 실제 whole33/source675/1670PASS·0FAIL·42ignored·43harness다. Root가 pure9와 원15step/8Cargo/7capture AST 불변 및 독립 최종검토를 확인했다. 이 문서 시점의 prepare/export/Cwrite/CI는 아직0이며 최종 문서 freeze 뒤 새로 실행한다. current32 namespace는 과거 작업명이고 whole33 명시 admission을 사용한다.

## Root 실제 준비·실행·독립검토

`prepare_root.py --proof WHOLE33 --root-review ROOT33 --volume-proof NATIVE_C_D --knownfolder-proof LARGE_BYTE_NATIVE` 실제0, export2128/config2129/originalsupply4897/runtime38을 새로고정했다. `run_root.py --proof WHOLE33 --root-review ROOT33 --root-approved-c-owned-temp-write` 실제0: Windows MSVC15단계/8Cargo/7capture 모두자연0·Job0,1670PASS/0FAIL/42ignored/43harness. `review.py --ci-terminal-confirmed`도 실제0으로 원675source/3PE/lock·전체원입력·원45step/capture/현재matrix/소유Croot와storage env를 독립 재대조했다. [Root 검토](../validation/evidence/current-ui-feedback-v32-local-ci-lowstorage-v4-20261016/root-review.json). namespace current32는 과거작업명이며 실제admission은whole33이다.

새Croot는사용자KnownFolder Temp의 nonce전용폴더이며보유parent/root nativeID·SID ACL·원marker를확인했다. runtime free/watermark를지켰고원7200초/기준변경0,최종원산출물과소유Ccache/target은보존했다. Cfree관측은volume단위이며실제CI전용peak나filesystemquota로주장하지않는다. 기존원CI2 OS112와CI3미실행/출력충돌은보존한다. 자동승인거절없음,사용자기존데이터삭제/시스템권한변경0.

CI독립검토까지 authored/source동결을유지했고그뒤현재완료보고문서만갱신했다. 따라서CI는고정export snapshot의실제성공이며그후의보고문서는최종check_docs로별도확인한다. compile source675/Cargo/lock변경0. hostedCI/원.git정상화·공개릴리스/서명·updater완료를주장하지않는다.
