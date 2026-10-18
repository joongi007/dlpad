# 앱 업데이트 서명 운영

앱 검증키는 `crates/engine/src/app_update/trust.rs`와 `crates/engine/src/app_update/release-public-key.json`에 고정한다. 비공개키·DPAPI blob을 Git/Release/artifact에 넣지 않는다. 앱의 Ed25519 업데이트 검증과 Windows Authenticode는 별개다.

2026-10-18 생성한 운영키 ID는 `dlpad-release-20261018-v1`이다. 비공개키는 개발 폴더의 ignored `.local-tools/update-activation-20261018/local-signing/release-v1.dpapi`에 현재 Windows 사용자용으로 암호화해 보관했다. 같은 계정·원 Windows 보호키가 없으면 복구할 수 없으며 이 경로를 임시 빌드 산출물처럼 청소하지 않는다. 별도 안전한 백업·키 교체는 운영 결정이다. 평문 비공개키나 실제 계정 비밀의 외부 전송은 이번 로컬 공급과 구분한다.

## 로컬 서명

WSL/Linux에서 `scripts/sign_local_app_update.py --help`를 확인한다. 기존 pinned Windows Python의 경로·SHA256, DPAPI 파일, 같은 CI 원본의 installer ZIP, version, 단조 증가 sequence, preview 채널, 발급·만료 Unix 시각, 비어 있는 출력 경로를 지정한다. 유효기간은 최대31일이며 통상14일을 사용한다. 평문은 로컬 파이프로 받아 /tmp의 전용0700 디렉터리·0600 파일에서 사용 후 제거한다. 키 bytes를 출력하거나 예외에 포함하지 않는다.

출력은 `update-windows-x86_64.json`, `.json.sig`, `update-public-key.json` 세 공개 파일이다. 공개키 파일은 배포 측 대조용이며 앱이 다운로드해서 신뢰하는 키가 아니다. 기존 `scripts/sign_app_update.py`가 원 ZIP·7개 member·바인딩·원 키와 생성 서명의 검증을 수행한다.

GitHub `release-signing` 환경의 자동 서명 job은 별도로 비밀키 공급을 승인·설정하기 전까지 비활성이다. 로컬 서명 후 공개 metadata만 해당 버전의 draft에 추가할 수 있다. 서명되지 않은 릴리스는 앱에서 수동 설치 안내를 유지한다. 만료된 새 후보도 실행하지 않는다. 원 공개 릴리스를 덮어쓰거나 기존 태그를 이동하지 않는다.

## 검증 범위

키 생성·로컬 DPAPI roundtrip과 공개키 추출을 실제 실행했다. 비공개키 외부 전송은 하지 않았다. 새 버전의 실제 설치·전환·복귀와 운영 metadata 공급 검증은 [진행 작업](work-items/S11-update-activation-20261018.md)에 기록한다.
