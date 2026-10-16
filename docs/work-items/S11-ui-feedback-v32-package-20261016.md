# UI 패키지: source32 후보 검수를 위한 namespace retarget

- 상태: proposed — 준비 완료, 실행 전
- 분류: validation/package preparation
- 우선순위: P0 UI 검수 후속
- 요구사항: R08 및 설치/배포 검수 계약
- 담당/수정 범위: field_reveal; ui-feedback-v32-package-v1 scratch의 기존12 code 파생본·argv·pins·handoff만
- 선행 작업: Root source32 Live 통합 및 actual whole32/독립 root-review 완료

## 사용자 결과

UI 개선 source32가 실제 전체 검증된 뒤 최신 portable/설치 후보를 같은 기존 파이프라인으로 만들고 검토할 수 있게 준비한다. 패키지 생성/설치 테스트를 실행한 것으로 보고하지 않는다.

## 계약과 범위

원 ui-feedback-v31-package-v1의12 code(상위8 Python, 하위3 Python+1 C#)를 읽어 namespace v31→v32만 바꿨다. 날짜20261016은 그대로다.5 code만 namespace가 있었고 나머지7 code는 원byte 동일이다. 전체12 code를 역치환한 bytes가 원v31과 정확히 동일하다. base-pins.json도 원v31 byte 그대로다. 원 helper/proof/ZIP/source/PE/Journal에는 변경하지 않는다. 과거 portable/package/native PASS 보고서는 새 경로로 복사하지 않는다.

실행 순서는 원7단계 graph→candidate→portable-review→generate→installer-zip→review→independent_review(--root-package-terminal-pass) 그대로다. dispatch는 미래 whole32의 실제 sibling Root review.source_entries를 읽는다. gate는 current source snapshot/Root proofSHA/raw4ops/실제43 harness totals/current3PE를 대조한다. retained harness/선택 수 같은 기존 guard는 future actual 측정값이 아니며 예상 합계/hash를 넣지 않았다.

archive exact7·streamed fullCRC/SHA·deterministic metadata·aggregate/member 상한·graph/notice/generated7/payload5 binding, source/current-proof afterguard, 원 private install/reinstall/installedCPU Job/naturalexit/사용자 input desktop 불변/oldjournal 보존 계약은 namespace 역치환 codebyte 동일로 유지한다. sign/legal/public release/upgrade/cleanOS/물리UI/전체P0 완료를 추가 주장하지 않는다.

## 인수 조건

- 원12 code의 v32→v31 역치환이 정확히 원bytes이고 원v31 실제 candidate pins가 맞는다.
- Python11 code는 ast.parse를 통과한다. C#compile/Cargo/package/native 실행은 하지 않는다.
- argv는 원7단계와 동일한 source32 exact namespace/fresh output으로만 향한다. 원 ZIP/PE/source를 덮지 않는다.
- Root가 actual whole32와 sibling root-review가 완료됐음을 확인한 뒤 실행한다. 출력값/PE SHA/source수/test수/native 완료값을 미리 예측하지 않는다.

## 검증과 handoff

실행한 것은 Python 정적 준비/AST/byte역치환/원 candidate-pin 대조뿐이다. static-result.json에12개 실제 파일hash/bytes와치환개수/각동일여부를 기록했다. Cargo/package/native 실행0. Root source32 proof경로가 준비한namespace와 다르면 원후보를 덮지 말고 새namespace를 조정한다.

비용: 제품/의존성/빌드/상주 비용 변경0. 동일12개 helper source와 작은 metadata 파생본만 새 scratch에 기록한다. 패키지 크기는 실제 실행 전 미측정이다.

Root가 실행 전 작업범위를 기록했다. 기존v31의 동일7단계/runtime proof gate를 유지하며 새로운whole32/3PE 완료전 패키지생성0이다. Cargo/native/패키지 실행은 Root가순차조정하고 직접통합검증한다.

Root 실제같은7단계·독립archivegraphreview 완료: portable18,333,239B/467members·installer17,282,743B/7members의전streamCRC/SHA·selectedgraph258/selectednotice누락0/원456notice를검증했다. 실제private설치2·제거·동일경로새journal재설치·cold검증·installedCPU1실행은자연0/Job0이며Root33pins/old-newjournal각5/payload5/3원PNG직접대조완료다. 공개/서명/legal전체/upgrade/physical/cleanOS/P0전체인수는아니다. [현재안내](../development/windows-user-preview.md).
