# ADR-0012: 서명된 앱 업데이트 후보와 설치 권한 분리

- 상태: Proposed, 검증/저장/조회 경계 구현·Windows 회귀 통과, 운영 교체 미완료
- 날짜: 2026-10-18
- 관련: [설치 기반](0005-app-install-foundation.md), [설치/앱 수명](0009-installer-launch-supervision.md), [업데이트 작업](../work-items/S11-app-updater.md)

## 배경

사용자가 소개 페이지 → 설치형/포터블 자동 릴리스 → 앱 업데이트를 지정했다. 현재 설치 API는 새로운 폴더에 개발 payload를 설치하는 기능이며, 실행 중인 앱 교체나 rollback 구현이 아니다. 설치 Supervisor의 소유 Job 아래에 helper를 단순 실행하면 앱 종료와 함께 helper도 종료될 수 있다.

## 구현한 경계

`engine::app_update`는 UI/설치 정책과 분리한다. 정확한 원 metadata bytes의 Ed25519 서명을 caller가 고정한 키로 먼저 검증한 뒤 JSON을 해석한다. 키는 후보 서버에서 가져와 신뢰하지 않는다. 중복/미지 필드, 낮거나 같은 버전·sequence, 만료, 다른 채널·target·저장소·URL·파일명, 잘못된 크기/해시를 거절한다. `ring=0.17.14`는 기존 rustls의 동일 잠금 버전을 직접 재사용하며 새 crate 버전은 추가하지 않는다. 실제 PE 증가 비용은 새 hosted 산출물로 측정하며, 모듈 추가를 자동 업데이트 활성화로 표시하지 않는다.

payload schema1의 domain은 `dlpad.app-update.v1`, target은 `x86_64-pc-windows-msvc`, repository는 `joongi007/dlpad`다. version은 선행0 없는 숫자3부분, sequence는 단조 증가 u64, channel은 stable/preview다. key_id, asset_name, asset_url, asset_bytes, asset_sha256, issued_at_unix, expires_at_unix를 모두 서명한다. 수명은 최대31일, payload는64KiB, 패키지는256MiB, 스트림 버퍼는64KiB다. signature는64byte를128자리 소문자 hex로 전달한다. Ed25519 서명은 Windows Authenticode나 법적 배포 완비를 뜻하지 않는다.

검증된 후보는 파일 생성·실행 권한이 아니다. 명시적인 패키지 저장은 기존 사용자 선택 parent의 원 HANDLE과 canonical 경로에서 새 staging 파일을 create-only로 만들고 flush/sync, 원 reader hash, identity를 확인한다. 기존 최종 파일을 덮어쓰지 않는 publication 뒤 다시 hash/identity를 확인하고 readonly reader·parent pin을 보유한다. 실패 정리는 원 created capability만 사용하며 불명확한 publication을 성공/삭제 권한으로 바꾸지 않는다. 이 저장 경로는 Windows 전용이며 다른 OS에서는 미지원이다. 사용자 미디어·외부 도구·이전 앱 설치를 변경하지 않는다.

## 남은 인수와 결정

운영 trust key 고정/보관·교체, signed metadata 자동 생성, native 설정의 확인/다운로드/오류·취소 UI, owned worker 수명, 설치기 전환, 기존 버전 보존·실패 복구를 완성해야 한다. 기존 journal generation/suffix는 업데이트 revision이 아니다. 실제 공개와 비밀 외부 전송은 사용자 지정 별도 승인 범위를 유지한다. 운영 키 없이 임의의 후보 키나 테스트 키를 기본 신뢰하지 않는다.

## 현재 검증

Root 실제 Windows MSVC의 engine all-targets strict Clippy 통과, metadata8+저장 실패회귀6 총14 PASS. 원 Job 자연종료/강제정리0/입력 desktop 불변을 확인했다. Linux의 verifier8+Windows 저장 미지원1 총9 PASS이며 기존 media_preview.rs의 Linux unused_mut 경고는 별도다. 전체 workspace/새 HTTP/실제 공개·업데이트 전환은 이 국소 시험으로 검증 완료 처리하지 않는다.


2026-10-18 통합 갱신: 공식 HTTPS 전송/공개 버전 discovery와 read-only native 설정을 추가했다. Root Windows workspace strict lint·전체 test(주1728+filtered child1 PASS/0FAIL/43ignored)·release build를 통과했다. 서명 후보가 설치 권한이 아니라는 경계는 유지한다. 운영키·signed 자동 생성의 실제 실행·owned helper 인계·교체/복구 인수는 남는다. Linux targeted27PASS를 전체 OS 지원으로 해석하지 않는다.


원 StagedPackage를 읽는 bundle 검사도 같은 권한 경계를 따른다. exact7 regular ZIP member/current profile, manifest5개의 version/target/hash·CRC·physical ranges·binding CPU/installer를64KiB stream/각 metadata64KiB/expanded256MiB로 제한한다. 원HANDLE 검증과 전체signedSHA 전후 검사를 통과해도 readonly BundleObservation만 반환한다. 현재profile 밖 ZIP64/extra/comment/preamble/trailing은 거절한다. 후보가 만료/다른 설치로 바뀌지 않았는지와 실행 권한은 미래 교체 coordinator에서 별도 확인해야 한다. Root Windows engine strict Clippy 및 app_update43PASS(실제stagecap1 포함), Linux workspace targeted37PASS, 실제49c unsigned installer ZIP parser 호환성을 확인했다. 운영키·교체/rollback 전체가 미완료이므로 본 ADR의 Proposed 상태는 유지한다.
