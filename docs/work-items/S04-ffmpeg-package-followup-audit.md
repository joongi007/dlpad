# S04: FFmpeg package 후속 공급·형식 감사

- 상태: done
- 분류: review
- 우선순위: P0
- 요구사항: R03, R05, R11
- 담당/수정 범위: metrics_recovery — 이 문서만
- 선행 작업: INTEGRATIONS/기존 고정 공급 실증 및 현재 ZIP package APIs

## 확보된 원본과 독립 재확인

기존 .local-tools/ffmpeg-supply-cnu82ffa/ffmpeg-n9.0.2-17-g2a571b6068-linux64-lgpl-9.0.tar.xz를 read-only streaming hash로 재확인했다. 실제137863452B/SHA2d41cbea0ca1a15029b638330740f78d6f5aeb062355bf425433fc938705350a다. 같은 소유 폴더 selected/bin/ffmpeg는142534568B/SHA953632dece404bb1461a4a2285f0191a6105d72c81f9da577eb54326a7fad49a, ffprobe는142305128B/SHAc0d4ee234730e7b083cc729108ac34f3b718cc4858c3cf8ed6e0dcec545f13c1, selected/LICENSE.txt는7651B/SHAda7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768다. 기존 문서 값과 일치한다. 다운로드·실행·재추출은 하지 않았다.

[기존 실제 검증](../validation/ffmpeg-supply-validation.md)은73 archive entries/선언450209477B, 선택 두binary+license284847347B(271.65MiB)를 기록한다. version n9.0.2-17-g2a571b6068-20260930, enable-version3/disable-libx264/libx265/enable-libopenh264, 실제 LGPL3-or-later다. 합성 MPEG4/AAC encode/probe/decode/기본 정확 자르기 통과는 당시 Linux 고정 후보 증거다. 현재 package installer와 Windows/macOS 전체 증거가 아니다.

## 현재 ZIP API와 즉시 호환성

현재 ArchivePackage는 flat component8개, ZIP directory32개, expanded 전체256MiB를 제한한다. 확보된 Linux tar.xz는 ZIP codec이 아니고73 nested members이며 선택 세트만도256MiB를 초과한다. 따라서 원 archive를 기존 API로 바로 설치할 수 없다. ZIP로 임의 재포장한 로컬 데이터를 공식 originalarchive 공급으로 취급하거나 상한을 조용히 올리면 안 된다.

고정 release.json에는 Windows LGPL ZIP ffmpeg-n9.0.2-17-g2a571b6068-win64-lgpl-9.0.zip171535354B가 있으나 실제 ZIP은 이번 보관 경로에서 찾지 못했다. metadata 후보만으로 member shape/component digest/notice/expanded fit를 판정할 수 없다. 일반 BtbN ZIP nested bin 경로를 flat현재계약으로 해석하는 변경 역시 실제 inventory 후 필요하다. 현재 확보된 원 Linux 후보를 지원하려면 bounded XZ decoder+stream TAR parser/정확 원member→flatcomponent mapping/디렉토리·비선택 regular entry 정책과 expansion capacity 새 계약이 필수다. 외부 tar 프로세스/extractall/전체memoryread 우회는 도입하지 않는다.

## trust·license 잔여

공식 FFmpeg 페이지가 BtbN을 안내하고 tag API와 checksums SHA가 일치하나 독립 실행 파일 서명·attestation 검증은 없다. archive에는 LICENSE.txt만 있고 대응 source/build recipe/dependency notice 완비를 입증하지 못했다. LGPL 파일명은 기본 LGPL2.1 또는 모든 의무 충족 근거가 아니다. FFmpeg source 서명은 외부 공급 binary 서명과 다르다. H264/yt-dlp 병합·정확 자르기/모든codec/minimumOS/Windows/macOS·관리versionselection·license사용자고지는 별도 gate다. Proposed 공급을 사용자 채택으로 승격하지 않는다.

## 다음 구체 수직작업

1. 원 고정 공급 format/capacity contract: Linux 원 tar.xz 기준으로 selected271.65MiB와 원archive+expanded403.13MiB를 실제 고정 reservation으로 설명하고 global managedcache/동시temp budget을 유지한다. bounded XZ dictionary/memory 및 TAR totalentries/totaldeclared/selectedactual/tail/truncation/duplicate/link/special/path mapping 실패 회귀와 크기·RSS 측정 후 codec 의존성/상한을 root ADR·Cargo로 조정한다. 범용 codec 확대는 하지 않는다.
2. 원 실제 archive를 native installer→inactive package→두 component 정확version/posthash→명시 선택→비활성 전체 제거로 연결하고 이전active/source/foreign 보존 인수를 수행한다. mainffmpeg+auxffprobe+license를 같은 원 record로 관리한다. metadata/download ffmpeg argv와 ffprobe 진단 기록은 별도 root 연결이다.
3. Windows ZIP은 별도 owned 공급 실증에서 archive checksum/member inventory/component hash/license/size를 먼저 확보하고 같은 고정mapping 계약으로 현재 boundedZIP 재사용 가능 여부를 판단한다. Linux tar.xz 해결을 Windows 검증으로 대체하지 않는다.
4. 실제 도구 UI는 기존 공급 선택/크기·license·미지원gate UX에 연결한다. 임의manifest 업로드 화면으로 범위를 축소하지 않는다. trust/source/notice 준비 전 설치 버튼을 출시채택으로 표시하지 않는다.

## 검증과 handoff

기존 archive/component의 read-only hash 및 저장 release metadata·코드·문서 조사만 수행했다. 새 다운로드·외부tool 실행·제품Rust/Cargo·설정 변경은 없다. 첫 후속구현은 기존Linux 후보를 위한 explicitformat/capacity gate와 bounded native tar.xz foundation이 필요하며 현재 ZIP installer 성공으로 표시할 수 없다.
