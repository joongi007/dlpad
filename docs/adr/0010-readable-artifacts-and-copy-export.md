# ADR-0010: 제목 파일명과 원본을 보존하는 복사 내보내기

- 상태: Proposed, Windows 격리 후보 검증 중
- 요구사항: R02, R08, R11
- 관련 작업: [제목 파일명](../work-items/S04-download-readable-filenames-20261013.md), [복사 내보내기](../work-items/S04-library-named-copy-export-20261013.md)

## 문제

job ID만 파일명으로 사용하면 파일 목록에서 영상을 알아보기 어렵다. 원본 이름을 직접 바꾸면 기존 다운로드·복구 계획과 파일 신원이 어긋날 수 있다.

## 후보 결정

새 다운로드는 선택적인 FinalNameStyle::TitleWithJobId를 저장하고 GUI 기본값은 제목 사용이다. 제목 뒤 full job ID를 붙여 출처를 구별한다. staging 이름과 원 외부 도구 argv는 job ID를 유지하고 최종 publication에만 이름 정책을 적용한다. StoredOptions의 version1에 생략 가능한 필드를 추가한다. 기존 missing/null은 legacy이며 기존 JSON·argv를 유지한다. 새 engine resume snapshot version2는 원 argv와 정책을 함께 검증한다. publication version3는 bounded 원 제목과 정규화된 stem의 binding을 재확인한다. 이전 version1/2 복구를 새 제목으로 재해석하지 않는다.

사용자가 이름을 지정해 가져가는 동작은 새 파일의 복사 내보내기로 분리한다. 기존 Library 선택의 root·full native ID·size·mtime를 worker에서 다시 확인하고 기존 단일 worker·Stop/CloseWait를 재사용한다. 설정 경로나 문자열을 파일 소유권으로 취급하지 않는다.

## 실패와 소유권

출력은 기존 부모 아래 상대 FILE_CREATE로 만든 새 stage만 쓰기·정리 capability를 갖는다. 원 사용자 파일이나 기존 목적 파일은 덮어쓰거나 지우지 않는다. 공개 전에는 원 생성 HANDLE·identity·parent를 긍정적으로 확인한 경우에만 stage를 정리한다. NT 공개 호출 이후 결과가 불확실하면 stage/final 경로·identity 사실을 남기고 삭제하지 않는다. 이 사실은 복구·삭제 권한이 아니다. 공개된 파일 관측용 duplicate는 WRITE/DELETE 권한을 제거한다.

Windows 금지 문자·예약 이름·제어/bidi 문자·UTF-8/UTF-16 component와 전체 경로 한계를 검사한다. 완전한 Unicode grapheme 정책은 아직 인수하지 않았다. 동일 이름 충돌은 실패하며 기존 파일을 보존한다.

## 비용과 검증 한계

복사는64KiB chunk, 최대1TiB, 협조적300초 deadline과 진행 mailbox1을 쓴다. source→stage, stage 검증, source 재검증, final 검증의 네 번 읽기/해시 비용이 있다. 물리 I/O나 동기 OS 호출의 hard deadline을 보증하지 않는다. 생성 capability·export의 실제 Windows scoped15/0, naming15/0와 UI context/picker/Stop 회귀는 원자료에 기록했다. 당시 v21 제품/설치본에는 이 후보가 포함되지 않았다. 현재 v22에는 통합되어 Windows 전체 검사 1,524/0/42와 합성 데이터 기반 실제 engine API 10개 시험을 통과했다. 제목 유지 다운로드·한글 복사·동명 대상 거절·원본 보존의 [독립 검토](../validation/evidence/current22-readable-copy-api-run-v5-20261014/root-review.json)를 남겼다. 실제 private GUI 복사 흐름은 별도 진행 중이며 물리 입력·폴더 picker·사용자 UI 인수는 아직 완료하지 않았다. 이 ADR의 전체 결정은 Proposed를 유지한다.
