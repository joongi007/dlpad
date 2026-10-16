# S04: FFmpeg license/source/notice 채택 조건 감사

- 상태: completed (로컬 artifact 감사; 공급 채택 미완료)
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R11
- 담당/수정 범위: soak_recovery_audit — 이 문서만
- 선행 작업: 고정 BtbN 후보 archive, 실제 추출/license 관측, package fixture

## 범위

기존 로컬 문서, 공급 metadata, 추출된 license와 실행 관측 text만 읽었다. 새 다운로드·Rust/Cargo 변경·도구 실행·외부 게시·공개 릴리스는 하지 않았다. 이 기록은 실제 artifact 준비 차이를 구분하는 개발 감사이며 법률 검토나 공급 채택 확정이 아니다.

## 실제 확보된 artifact

후보는 BtbN tag `autobuild-2026-09-30-13-08`의 Linux64 LGPL `ffmpeg-n9.0.2-17-g2a571b6068-linux64-lgpl-9.0.tar.xz`이다. [실제 native 추출](../validation/evidence/ffmpeg-native-tar-xz-first/result.json)과 [package fixture](../validation/evidence/ffmpeg-v2-package-fixture/package.json)는 archive 137863452 bytes/SHA256 `2d41cbea0ca1a15029b638330740f78d6f5aeb062355bf425433fc938705350a`를 고정한다.

| 추출 artifact | bytes | SHA256 |
|---|---:|---|
| ffmpeg | 142534568 | 953632dece404bb1461a4a2285f0191a6105d72c81f9da577eb54326a7fad49a |
| ffprobe | 142305128 | c0d4ee234730e7b083cc729108ac34f3b718cc4858c3cf8ed6e0dcec545f13c1 |
| LICENSE.txt | 7651 | da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768 |

로컬 selected/LICENSE.txt의 실제 hash도 위 값과 동일했다. 내용은 LGPL version 3 전문이며 GPL version 3의 조건을 편입한다는 문장이 있다. LGPL 전문 한 파일이 GPL 전문이나 모든 dependency notice를 대신한다고 해석하지 않는다. 실행 관측 `ffmpeg-license.txt`/`ffprobe-license.txt`는 `n9.0.2-17-g2a571b6068-20260930`과 LGPL3-or-later를 보고한다. build configuration에는 enable-version3, disable-libx264/libx265 및 여러 정적 dependency가 표시된다. 파일명 LGPL만으로 전체 dependency 권리 목록을 증명하지 않는다.

[관리 inactive 설치 시험](../validation/evidence/ffmpeg-v2-managed-install-first/result.json)은 install/remove와 source hash 보존 PASS이나 `supply_adopted=false`이다. package fixture license 문구도 corresponding source와 third-party notice adoption pending을 명시한다. LICENSE를 Notice role로 설치할 수 있다는 schema 지원은 notice 완비 인수가 아니다.

## 구체적으로 남은 자료와 결정

- exact FFmpeg commit `2a571b6068` 및 이 공급 build에 대응하는 source archive/repository revision, build recipe/toolchain/configuration, enabled third-party dependency별 정확 revision과 license/notice mapping이 로컬 selected 자료에 없다. FFmpeg tag n9.0.2 일반 source 링크만으로 `-17-g...` 공급 bytes의 대응성을 완료 처리하지 않는다.
- LGPL3가 편입하는 GPL3 전문, dependency notices 및 copyright 자료를 release package의 bounded Notice component로 어떻게 보존·표시할지 고정하지 않았다. 현재 selected에는 두 실행 파일과 LICENSE.txt만 있다.
- 공급 checksum/tag metadata 대조와 독립 publisher signing/attestation trust는 다른 검증이다. 후보 archive checksum을 제품 자동 공급 채택으로 승격할 결정은 남는다.
- Windows/macOS 대응 build, 플랫폼별 license/source/notice·크기·runtime·codec 검증은 Linux 자료로 대체할 수 없다. macOS BtbN 자동 build 후보가 없다는 기존 문서 제한을 유지한다.
- [공급 문서](../validation/tool-supply-evidence.md)의 당시 archive 미다운로드/미추출 서술은 역사 기록이다. 이후 actual extraction/install/version evidence와 구분해 최신 진행 문서를 갱신하되 최초 원시 evidence를 덮어쓰지 않는다.

## 사용자 별도 승인 없이 진행 가능한 개발

고정 후보의 source/build/dependency notice 자료를 공개 공식 공급 위치에서 읽거나 프로젝트 임시 cache에 다운로드하고 hashes/revisions를 기록하는 작업, notice manifest와 native UI 표시 준비, bounded Notice component 설치·변조 거절 회귀, synthetic codec/runtime 시험, 공급 상태를 Proposed/unverified로 유지하는 문서 갱신은 이미 허용된 개발 범위다. 현재 task에서는 다운로드를 수행하지 않았다.

이 자료 수집 때문에 사용자 답변을 기다릴 이유는 없다. 다만 자료가 완비됐다는 근거 없이 자동 공급을 확정하거나 공개 릴리스 배포를 실행할 수는 없다. 실제 공개 릴리스·외부 게시·유료 서비스·관리자 설정 변경은 사용자 별도 승인 범위이며 해당 작업만 보류하고 독립 개발을 계속한다. 공급 선택은 Proposed 상태를 유지하고 근거가 모이면 주 담당이 채택 여부를 기록한다.

## 검증과 잔여

실행한 명령은 로컬 rg/cat/wc/sha256sum 읽기뿐이다. LICENSE 7651-byte hash 일치를 직접 확인했고 다른 component hash는 기존 actual extraction 원시 결과를 대조했다. 법적 의무 충족·notice 완비·source correspondence·3 OS 공급 채택은 미검증이다.
