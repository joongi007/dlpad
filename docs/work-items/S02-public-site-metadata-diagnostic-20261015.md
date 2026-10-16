# S00/S10: 두 공개 URL metadata 실패 단계의 bounded 진단

- 상태: proposed / 준비만
- 분류: verification
- 우선순위: P0
- 요구사항: R04, R08, R10; S00 media UI/S10 diagnostic
- 담당/수정 범위: search_reveal, B/p0-real-site-metadata-v2만. Root가 docs/work-items 채택, 실행, 통합 검증한다.
- 선행 작업: 현재 UI/whole28 게이트 종료, actual sibling Root review, 명시 yt-dlp/Deno tool proof.

## 사용자 결과

사용자가 제공한 YouTube·치지직 URL의 metadata 준비가 실패하면 어느 단계·exit/outcome인지와 정제된 stderr tail을 확인할 수 있도록 개발 진단을 준비한다. 이전 GUI 약3초 실패 원인은 현재 미확정이다.

## 계약과 범위

production engine media_preview API를 먼저 사용하고 실패 때만 process adapter metadata argv를 한 번 수집한다. managed execution/GUI와 다른 제한 증거임을 명시한다. 고정 query 없는2URL, 인증·쿠키·실계정 없음, 전체영상 다운로드 없음, 임시root·ownedJob/private desktop·argv·timeout/socket/retry·capture bound·production redactor·도구 원identity/SHA·현재 source/3PE 불변을 유지한다. external network는 Root가 명시 실행할 때만 사용한다. 외부 server/service 공개와 전역 변경은 없다.

## 인수 조건

Root가 실제 compile와 두 metadata API 실행 및 원 Job/source/도구 pin을 검토한다. 각각 engine status/실패stage·보조 exit/outcome·정제tail·정확한yt/Deno 버전·Deno/EJS 옵션을 읽는다. 원metadata·민감 query·rawstderr·cookie/auth secret은 직렬화하지 않는다. redactor 생략을 허용하고 원인 확정을 추정하지 않는다. 이전 증거는 보존하고 전체 GUI/managed session/도구 공급·P0 완료로 확대하지 않는다.

## 검증과 handoff

실제 명령은 argv.json이다. 준비 시 Python AST와 Rust1.94 rustfmt만 통과했다. Cargo/native/network 실행0, Live/S/docs/공유 Cargo/lock/schema 변경0. scratch manifest의 dependency는 프로젝트 기존 engine/core와 기존 serde_json/sha2뿐이며 제품에 새 의존성·thread·timer를 추가하지 않는다. 준비 실행 파일 크기·CPU 비용은 미측정이고 offline 빌드 실패 가능성은 남는다.

Root v2 build 실제0뒤 forward-slash target 경로 검증예외로 metadata미실행, 원실패보존. v3는PureWindowsPath 정규화와 원strict ownedpath검사를유지하고 v2실제lock을상속해 offline/locked build 및metadata 각각0/Job0를확인했습니다. Root188pins/current674source/3PE·원로그·JSON receipt를독립검토했습니다. YouTube준비(duration5722000ms), 치지직ToolFailed/제한보조Exit1·2줄생략이며원인미확정. Deno runtimeversion필터결과null이므로정상파싱됐다고주장하지않습니다. 쿠키/인증/영상다운로드/GUI성공·이전실패원인확정없음.
